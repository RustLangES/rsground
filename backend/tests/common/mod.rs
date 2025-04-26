use core::fmt;

use awc::http::header::TryIntoHeaderPair;

#[macro_export(local_inner_macros)]
macro_rules! _const {
    (API_URL) => {
        "http://localhost:8080"
    };
    (WS_URL) => {
        "ws://localhost:8080/ws"
    };
}

pub fn auth_header(token: impl fmt::Display) -> impl TryIntoHeaderPair {
    ("Authorization", format!("Bearer {token}"))
}

#[macro_export]
macro_rules! request {
    ($path:literal $($tt:tt)*) => { request!([get] $path $($tt)*) };
    ([$method:ident] $path:literal $($tt:tt)*) => {
        request!(@compose [::awc::Client::new().request(::awc::http::Method::$method, concat!(_const!(API_URL), $path))] $($tt)*)
    };
    (@compose [$base:expr]) => { $base };
    (@send [$base:expr] dbg $(, $($tt:tt)+)?) => {
        request!(@compose [{
            let response = $base;
            dbg!(&response);
            response
        }] $($($tt)+)?)
    };
    (@compose [$base:expr] as $auth:expr $(, $($tt:tt)+)?) => {
        request!(@compose [$base.insert_header($crate::common::auth_header($auth))] $($($tt)+)?)
    };

    (@compose [$base:expr] send $(, $($tt:tt)+)?) => {
        request!(@send [$base.send().await.unwrap()] $($($tt)+)?)
    };

    (@compose [$base:expr] send { $($body:tt)* } $(, $($tt:tt)+)?) => {
        request!(@send [$base.send_json(&::serde_json::json!({ $($body)* })).await.unwrap()] $($($tt)+)?)
    };

    (@send [$base:expr]) => { $base };
    (@send [$base:expr] dbg $(, $($tt:tt)+)?) => {
        request!(@send [{
            let response = $base;
            dbg!(&response);
            response
        }] $($($tt)+)?)
    };
    (@send [$base:expr] expect $code:literal $(, $($tt:tt)+)?) => {
        request!(@send [{
            let response = $base;
            assert_eq!(response.status(), $code as u16);
            response
        }] $($($tt)+)?)
    };

    (@send [$base:expr] expect $code:ident $(, $($tt:tt)+)?) => {
        request!(@send [{
            let response = $base;
            assert_eq!(response.status(), ::awc::http::StatusCode::$code);
            response
        }] $($($tt)+)?)
    };

    (@send [$base:expr] json $(, $($tt:tt)+)?) => {
        json_assert!($base.json::<::serde_json::Value>().await.expect("Should be a json"), $($($tt)+)?)
    };

}

#[macro_export]
macro_rules! ws {
    (connect $owner:expr, $project_id:expr $(, $password:expr)?) => {
        ::awc::Client::new()
            .ws(format!(concat!(_const!(WS_URL), "/{}"), $project_id))
            .set_header("Sec-WebSocket-Protocol", ws!(@connect-protocol $owner $($password)?))
            .connect()
            .await
            .unwrap()
            .1
    };

    (@connect-protocol $owner:expr) => {format!("auth.{}", $owner)};
    (@connect-protocol $owner:expr, $password:expr) => {format!("auth.{}, password.{}", $owner, $password)};

    (recv $ws:ident, $name:literal $(, $($tt:tt)+)?) => {
        match ::tokio::time::timeout(::core::time::Duration::from_millis(100), $ws.next()).await {
            Ok(Some(Ok(awc::ws::Frame::Text(msg)))) =>  {
                json_assert!(
                    ::serde_json::from_slice::<::serde_json::Value>(&msg).unwrap(),
                    tee_ref
                    [ get "action", as string, eq $name ]
                    $($($tt)+)?
                )
            }
            Ok(Some(Ok(kind))) => panic!(concat!("Expecting Text. Should receive ", $name, ": {:?}"), kind),
            Ok(Some(Err(err))) => panic!(concat!("Error receiving message. Should receive ", $name, ": {}"), err),
            Ok(None) => panic!(concat!("No remaining messages. Should receive ", $name)),
            Err(_) => panic!(concat!("Timeout. Should receive ", $name)),
        }
    };

    (send $ws:ident, $action:literal { $($tt:tt)* }) => {
        $ws
            .send(::awc::ws::Message::Text(::serde_json::json!({
                "action": $action,
                $($tt)*
            }).to_string().into()))
            .await
            .unwrap()
    };
}

#[macro_export]
macro_rules! json_assert {
    ($base:expr $(, $($tt:tt)+)?) => { json_assert!(@json (Value) [$base] $($($tt)+)?) };

    (@json [$base:expr] $($tt:tt)*) => { json_assert!(@json (Value) [$base] $($tt)*) };
    (@json ($ty:ident) [$base:expr]) => { $base };
    (@json ($ty:ident) [$base:expr] tee_ref $([ $($tt:tt)+ ])+) => {{
        let response = $base;

        ($(
            json_assert!(@json ($ty) [&response] $($tt)+),
        )+)
    }};
    (@json (Value) [$base:expr] get $key:literal $(, $($tt:tt)+)?) => {
        json_assert!(@json (Value) [$key] [$base.get($key).expect(&format!(concat!('"', $key, "\" should exist in {}"), $base))] $($($tt)+)?)
    };
    (@json (Value) [$base:expr] get $key:literal $(, $($tt:tt)+)?) => {
        json_assert!(@json (Value) [$key] [$base.get($key).expect(&format!(concat!('"', $key, "\" should exist in {}"), $base))] $($($tt)+)?)
    };

    (@json ($ty:ident) [$base:expr] eq $eq:expr $(, $($tt:tt)+)?) => {
        json_assert!(@json ($ty) [{
            let base = $base;
            assert_eq!(base, $eq);
            base
        }] $($($tt)+)?)
    };
    (@json ($ty:ident) [$base:expr] dbg $(, $($tt:tt)+)?) => {
        json_assert!(@json ($ty) [{
            let base = $base;
            println!(concat!("[", file!(), ":", line!(), ":", column!(), "] {:?}"), base);
            base
        }] $($($tt)+)?)
    };

    (@json ($ty:ident) [$key:literal] [$base:expr]) => { $base };

    (@json ($ty:ident) [$key:literal] [$base:expr] eq $eq:expr $(, $($tt:tt)+)?) => {
        json_assert!(@json ($ty) [$key] [{
            let base = $base;
            assert_eq!(base, $eq, $key);
            base
        }] $($($tt)+)?)
    };

    (@json ($ty:ident) [$key:literal] [$base:expr] dbg $(, $($tt:tt)+)?) => {
        json_assert!(@json ($ty) [$key] [{
            let base = $base;
            println!(concat!("[", file!(), ":", line!(), ":", column!(), "] ", $key, " = {:?}"), base);
            base
        }] $($($tt)+)?)
    };

    (@json (Array) [$key:literal] [$base:expr] expect empty $(, $($tt:tt)+)?) => {
        json_assert!(@json (Array) [$key] [{
            let base = $base;
            if !base.is_empty() {
                panic!(concat!('"', $key, "\" should be empty"))
            }
            base
        }] $($($tt)+)?)
    };

    (@json (Value) [$key:literal] [$base:expr] as array $(, $($tt:tt)+)?) => {
        json_assert!(
            @json
            (Array)
            [$key]
            [$base
                .as_array()
                .expect(&format!(concat!('"', $key, "\" should be an array")))
                .clone()]
            $($($tt)+)?
        )
    };

    (@json (Value) [$key:literal] [$base:expr] as unsigned $(, $($tt:tt)+)?) => {
        json_assert!(
            @json
            (Unsigned)
            [$key]
            [$base
                .as_u64()
                .expect(&format!(concat!('"', $key, "\" should be an unsigned number")))]
            $($($tt)+)?
        )
    };

    (@json (Value) [$key:literal] [$base:expr] as string $(, $($tt:tt)+)?) => {
        json_assert!(
            @json
            (String)
            [$key]
            [$base
                .as_str()
                .expect(&format!(concat!('"', $key, "\" should be a string")))
                .to_string()]
            $($($tt)+)?
        )
    };

    (@json (Value) [$key:literal] [$base:expr] as object $(, $($tt:tt)+)?) => {
        json_assert!(
            @json
            (Value)
            [$base
                .as_object()
                .expect(&format!(concat!('"', $key, "\" should be an object")))
                .clone()]
            $($($tt)+)?
        )
    };
}
