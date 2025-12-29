/*
syntax: execute(Rd1&Rc2|Re3)
execute(!Rd(0..5))
execute(Rd4&(Ra2^Ra3))
*/
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::{Parse, ParseStream}, parse_macro_input, ExprLit, ExprRange, Ident, Lit, Result, Token};

/// Logical / syntax tokens
enum SyntaxToken {
    And(Token![&]),
    Or(Token![|]),
    Xor(Token![^]),
    Not(Token![!]),
    Eq(Token![=]),
    Ne(Token![!=]),
    Group(Vec<SyntaxToken>),
    Var(Variable),
}

impl ToTokens for SyntaxToken {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let data = match self {
            SyntaxToken::And(_) => Some(quote! {&}),
            SyntaxToken::Or(_) => Some(quote! {|}),
            SyntaxToken::Xor(_) => Some(quote! {^}),
            SyntaxToken::Not(_) => Some(quote! {!}),
            SyntaxToken::Eq(_) => Some(quote! {==}),
            SyntaxToken::Ne(_) => Some(quote! {!=}),
            SyntaxToken::Group(t) => {
                let mut inner = TokenStream::new();
                t.iter().for_each(|t| t.to_tokens(&mut inner));
                Some(quote! { (#inner) })
            }
            SyntaxToken::Var(t) => {
                t.to_tokens(tokens);
                None
            }
        };
        if let Some(d) = data {
            tokens.extend(d);
        }
    }
}

/// Variable like Rd1, Rd(0..5), Rd(*)
struct Variable {
    name: Ident,
    expr: VariableExpr,
}

impl ToTokens for Variable {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        // CHANGE: We prepend `&` to #name (e.g., (&ra)) so we always pass a reference
        // to extract_val. This prevents moving the variable.
        match &self.expr {
            VariableExpr::All => {
                tokens.extend(quote! { (&#name).extract_val()? });
            }
            VariableExpr::Literal(lit) => {
                tokens.extend(quote! { ((((&#name).extract_val()? >> #lit) & 1) != 0) });
            }
            VariableExpr::Range(_) => {
                tokens.extend(quote! { compile_error!("Ranges are not yet implemented") });
            }
        }
    }
}

enum VariableExpr {
    Literal(ExprLit),
    Range(ExprRange),
    All,
}

struct Syntax {
    tokens: Vec<SyntaxToken>,
}

impl Parse for Syntax {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut tokens = Vec::new();

        while !input.is_empty() {
            // ... (Same parsing logic as before) ...
            if input.peek(Token![&]) {
                tokens.push(SyntaxToken::And(input.parse()?));
            } else if input.peek(Token![|]) {
                tokens.push(SyntaxToken::Or(input.parse()?));
            } else if input.peek(Token![^]) {
                tokens.push(SyntaxToken::Xor(input.parse()?));
            } else if input.peek(Token![!]) {
                tokens.push(SyntaxToken::Not(input.parse()?));
            } else if input.peek(Token![!=]) {
                tokens.push(SyntaxToken::Ne(input.parse()?));
            } else if input.peek(Token![=]) {
                tokens.push(SyntaxToken::Eq(input.parse()?));
            } else if input.peek(syn::token::Paren) {
                let content;
                syn::parenthesized!(content in input);
                let inner = content.parse::<Syntax>()?;
                tokens.push(SyntaxToken::Group(inner.tokens));
            } else if input.peek(Ident) {
                let mut name: Ident = input.parse()?;
                let mut expr = VariableExpr::All;

                // Handle fused identifiers like "a7"
                let name_str = name.to_string();
                if let Some(digit_pos) = name_str.find(|c: char| c.is_ascii_digit()) {
                    let (var_part, num_part) = name_str.split_at(digit_pos);
                    if !var_part.is_empty() && num_part.chars().all(|c| c.is_ascii_digit()) {
                        name = Ident::new(var_part, name.span());
                        let lit_int = syn::LitInt::new(num_part, name.span());
                        expr = VariableExpr::Literal(ExprLit { attrs: vec![], lit: Lit::Int(lit_int) });
                    }
                }

                if matches!(expr, VariableExpr::All) {
                    if input.peek(syn::LitInt) {
                        expr = VariableExpr::Literal(input.parse()?);
                    } else if input.peek(syn::token::Paren) {
                        let content;
                        syn::parenthesized!(content in input);
                        if content.peek(Token![*]) {
                            content.parse::<Token![*]>()?;
                            expr = VariableExpr::All;
                        } else {
                            let range: ExprRange = content.parse()?;
                            expr = VariableExpr::Range(range);
                        }
                    }
                }
                tokens.push(SyntaxToken::Var(Variable { name, expr }));
            } else {
                return Err(input.error("unexpected token in execute! macro"));
            }
        }
        Ok(Syntax { tokens })
    }
}

#[proc_macro]
pub fn execute(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let data = parse_macro_input!(input as Syntax);

    let mut body_stream = TokenStream::new();
    data.tokens.iter().for_each(|x| {
        x.to_tokens(&mut body_stream)
    });

    // We implement __BinExprExtract for REFERENCES (&Result, &u8, etc.)
    // to avoid moving ownership.
    let expanded = quote! {
        {
            trait __BinExprExtract {
                type Output;
                fn extract_val(self) -> ::anyhow::Result<Self::Output>;
            }

            // === Primitive Implementations (Reference Handling) ===
            // Handles: let a: u8; execute!(a); -> &a passed
            impl<'a> __BinExprExtract for &'a u8 {
                type Output = u8;
                fn extract_val(self) -> ::anyhow::Result<u8> { Ok(*self) }
            }
            // Handles: let a: &u8; execute!(a); -> &a passed (becomes &&u8)
            impl<'a> __BinExprExtract for &'a &'a u8 {
                type Output = u8;
                fn extract_val(self) -> ::anyhow::Result<u8> { Ok(**self) }
            }
            // Handles: let a: &mut u8; execute!(a); -> &a passed (becomes &&mut u8)
            impl<'a> __BinExprExtract for &'a &'a mut u8 {
                type Output = u8;
                fn extract_val(self) -> ::anyhow::Result<u8> { Ok(**self) }
            }

            impl<'a> __BinExprExtract for &'a bool {
                type Output = bool;
                fn extract_val(self) -> ::anyhow::Result<bool> { Ok(*self) }
            }
            impl<'a> __BinExprExtract for &'a &'a bool {
                type Output = bool;
                fn extract_val(self) -> ::anyhow::Result<bool> { Ok(**self) }
            }
            impl<'a> __BinExprExtract for &'a &'a mut bool {
                type Output = bool;
                fn extract_val(self) -> ::anyhow::Result<bool> { Ok(**self) }
            }

            // === Result Implementations (Reference Handling) ===
            // These take &Result<...> so they don't consume the Result.
            // If it's an Error, we clone/format it into a new anyhow::Error.

            // Handles: Result<u8, E>
            impl<'a, E> __BinExprExtract for &'a ::std::result::Result<u8, E>
            where E: ::std::fmt::Display
            {
                type Output = u8;
                fn extract_val(self) -> ::anyhow::Result<u8> {
                    match self {
                        Ok(v) => Ok(*v),
                        Err(e) => Err(::anyhow::anyhow!("{}", e)),
                    }
                }
            }

            // Handles: Result<&u8, E>
            impl<'a, E> __BinExprExtract for &'a ::std::result::Result<&'a u8, E>
            where E: ::std::fmt::Display
            {
                type Output = u8;
                fn extract_val(self) -> ::anyhow::Result<u8> {
                    match self {
                        Ok(v) => Ok(**v),
                        Err(e) => Err(::anyhow::anyhow!("{}", e)),
                    }
                }
            }

            // Handles: Result<&mut u8, E>
            impl<'a, E> __BinExprExtract for &'a ::std::result::Result<&'a mut u8, E>
            where E: ::std::fmt::Display
            {
                type Output = u8;
                fn extract_val(self) -> ::anyhow::Result<u8> {
                    match self {
                        Ok(v) => Ok(**v), // Read the value from the mutable reference
                        Err(e) => Err(::anyhow::anyhow!("{}", e)),
                    }
                }
            }

            // Handles: Result<bool, E>
            impl<'a, E> __BinExprExtract for &'a ::std::result::Result<bool, E>
            where E: ::std::fmt::Display
            {
                type Output = bool;
                fn extract_val(self) -> ::anyhow::Result<bool> {
                    match self {
                        Ok(v) => Ok(*v),
                        Err(e) => Err(::anyhow::anyhow!("{}", e)),
                    }
                }
            }

            // Handles: Result<&bool, E>
            impl<'a, E> __BinExprExtract for &'a ::std::result::Result<&'a bool, E>
            where E: ::std::fmt::Display
            {
                type Output = bool;
                fn extract_val(self) -> ::anyhow::Result<bool> {
                    match self {
                        Ok(v) => Ok(**v),
                        Err(e) => Err(::anyhow::anyhow!("{}", e)),
                    }
                }
            }
             // Handles: Result<&mut bool, E>
            impl<'a, E> __BinExprExtract for &'a ::std::result::Result<&'a mut bool, E>
            where E: ::std::fmt::Display
            {
                type Output = bool;
                fn extract_val(self) -> ::anyhow::Result<bool> {
                    match self {
                        Ok(v) => Ok(**v),
                        Err(e) => Err(::anyhow::anyhow!("{}", e)),
                    }
                }
            }

            (|| -> ::anyhow::Result<_> {
                Ok(#body_stream)
            })()
        }
    };

    expanded.into()
}