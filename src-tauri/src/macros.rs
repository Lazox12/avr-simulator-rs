#[macro_export]
macro_rules! wrap_anyhow {
    // Function with args
    ($name:ident ( $($arg:ident : $typ:ty),* ) -> $ret:ty $body:block) => {
        #[tauri::command]
        pub fn $name($($arg : $typ),*) -> ::tauri::Result<$ret> {
            let result = (|| -> ::anyhow::Result<$ret> {$body})();
            if(result.is_err()){
                println!("Error:{}, {:?}",stringify!($name), result);
            }
            else {
                println!("Success:{}",stringify!($name));
            }

            result.map_err(|e| ::tauri::Error::Anyhow(e))
        }
    };

    // Function with no args
    ($name:ident () -> $ret:ty $body:block) => {
        #[tauri::command]
        pub fn $name() -> ::tauri::Result<$ret> {
            let result = (|| -> ::anyhow::Result<$ret> {$body})();
            if cfg!(debug_assertions){
                if(result.is_err()){
                    println!("Error:{}, {:?}",stringify!($name), result);
                }
                else {
                    println!("Success:{}",stringify!($name));
                }
            }
            result.map_err(|e| ::tauri::Error::from(e.to_string()))
        }
    };
}
