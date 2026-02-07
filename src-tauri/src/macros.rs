#[macro_export]
macro_rules! wrap_anyhow {
    // Function with no args
    ($name:ident () -> $ret:ty $body:block) => {
        #[tauri::command]
        pub fn $name() -> ::tauri::Result<$ret> {
            let result = (|| -> ::anyhow::Result<$ret> {$body})();
            if cfg!(debug_assertions){
                if(result.is_err()){
                    println!("Error:({})->{:?}",stringify!($name), result);
                }
                else {
                    println!("Success:({})->{:?}",stringify!($name),result);
                }
            }
            result.map_err(|e| ::tauri::Error::Anyhow(e))
        }
    };

    // Function with args
    ($name:ident ( $($arg:ident : $typ:ty),* ) -> $ret:ty $body:block) => {
        #[tauri::command]
        pub fn $name($($arg : $typ),*) -> ::tauri::Result<$ret> {
            let args = format!("{:?}",$($arg.clone()),*);
            let result = (|| -> ::anyhow::Result<$ret> {$body})();
            if(result.is_err()){
                println!("Error:{}({:?})->{:?}",stringify!($name),args,result);
            }
            else {
                println!("Success:{}({:?})->{:?}",stringify!($name),args,result);
            }

            result.map_err(|e| ::tauri::Error::Anyhow(e))
        }
    };

    (async $name:ident () -> $ret:ty $body:block) => {
        #[tauri::command]
        pub async fn $name() -> ::tauri::Result<$ret> {
            let result = (async || -> ::anyhow::Result<$ret> {$body})().await;
            if cfg!(debug_assertions){
                if(result.is_err()){
                    println!("Error:({})->{:?}",stringify!($name), result);
                }
                else {
                    println!("Success:({})->{:?}",stringify!($name),result);
                }
            }
            result.map_err(|e| ::tauri::Error::Anyhow(e))
        }
    };

    // Function with args
    (async $name:ident ( $($arg:ident : $typ:ty),* ) -> $ret:ty $body:block) => {
        #[tauri::command]
        pub async fn $name($($arg : $typ),*) -> ::tauri::Result<$ret> {
            let args = format!("{:?}",$($arg.clone()),*);
            let result = (async || -> ::anyhow::Result<$ret> {$body})().await;
            if(result.is_err()){
                println!("Error:{}({:?})->{:?}",stringify!($name),args,result);
            }
            else {
                println!("Success:{}({:?})->{:?}",stringify!($name),args,result);
            }

            result.map_err(|e| ::tauri::Error::Anyhow(e))
        }
    };

}
#[macro_export]
macro_rules! emit {
    ($name:expr, $val:expr) => {{
        use tauri::Emitter;
        println!("emitting: {}, params: {:?}", $name, $val);

        crate::get_app_handle()?.emit($name, $val)?;
    }};
}
