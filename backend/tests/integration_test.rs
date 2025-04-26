mod common;

use futures_util::{sink::SinkExt, stream::StreamExt};

async fn login_as(guest_name: &str) -> (String, String) {
    request!([POST] "/auth/guest"
        send { "guest_name": guest_name },
        expect OK,
        json,
        tee_ref
        [ get "jwt", as string ]
        [ get "id", as string ]
    )
}

async fn create_project(token: &str) -> String {
    request!([POST] "/create/test_project"
        as token,
        send,
        expect CREATED,
        json,
        get "id",
        as string
    )
}

#[actix_rt::test]
async fn test_flow_two_users() {
    // --- 1. Log in for both users ---
    let (owner, owner_id) = login_as("owner").await;
    let (guest, guest_id) = login_as("guest").await;

    // --- 2. Create project as "owner" ---
    let project_id = create_project(&owner).await;

    // --- 3. Connect both websockets ---
    let mut owner_ws = ws!(connect owner, &project_id);

    // Owner handshake
    {
        let (_, users) = ws!(recv owner_ws, "welcome", [ get "users", as object ]);
        assert!(users.contains_key(&owner_id), "Self should be included")
    }

    // Owner get notified about its own connection
    _ = ws!(recv owner_ws, "user_connected", [ get "user_id", as string, eq owner_id ]);

    let mut guest_ws = ws!(connect guest, &project_id);

    // Guest handshake
    let guest_session = {
        let (_, users, session_id) = ws!(recv guest_ws, "welcome", [ get "users", as object ] [ get "session_id", as string ]);
        assert!(users.contains_key(&guest_id), "Self should be included");
        session_id
    };

    // Get notified about guest connection
    _ = ws!(recv guest_ws, "user_connected", [ get "user_id", as string, eq guest_id ]);
    _ = ws!(recv owner_ws, "user_connected", [ get "user_id", as string, eq guest_id ]);

    // --- 4. Gives editor to guest ---
    _ = ws!(send owner_ws, "permit_access" {
        "user_id": guest_id,
        "access": "editor"
    });

    // Access update
    _ = ws!(recv owner_ws, "update_access", [ get "user_id", as string, eq guest_id ] [ get "access", as string, eq "editor" ]);
    _ = ws!(recv guest_ws, "update_access", [ get "user_id", as string, eq guest_id ] [ get "access", as string, eq "editor" ]);

    // --- 5. Inserts text in "test" file ---
    _ = ws!(send guest_ws, "file_create" {
        "file": "test"
    });

    _ = ws!(recv owner_ws, "project_files", [ get "files", as array, eq [ "test" ] ] );
    _ = ws!(recv guest_ws, "project_files", [ get "files", as array, eq [ "test" ] ] );

    _ = ws!(send guest_ws, "sync" {
        "revision": 0,
        "file": "test",
        "actions": [{ "kind": "insertion", "from": 0, "text": "hello world", "owner": guest_session }],
    });

    _ = ws!(recv owner_ws, "sync", [ get "actions", as array ] [ get "file", as string, eq "test" ] [ get "revision", as unsigned, eq 1 ]);
    _ = ws!(recv guest_ws, "sync", [ get "actions", as array ] [ get "file", as string, eq "test" ] [ get "revision", as unsigned, eq 1 ]);

    // --- 6. Guest deletes text in "test" file ---
    _ = ws!(send guest_ws, "sync" {
        "revision": 0,
        "file": "test",
        "actions": [{ "kind": "deletion", "from": 0, "to": 5, "owner": guest_session }],
    });

    _ = ws!(recv owner_ws, "sync", [ get "actions", as array ] [ get "file", as string, eq "test" ] [ get "revision", as unsigned, eq 2 ]);
    _ = ws!(recv guest_ws, "sync", [ get "actions", as array ] [ get "file", as string, eq "test" ] [ get "revision", as unsigned, eq 2 ]);

    // --- 7. Guest deletes "test" file ---
    _ = ws!(send guest_ws, "file_delete" {
        "file": "test"
    });

    _ = ws!(recv owner_ws, "project_files", [ get "files", as array, dbg, expect empty ] );
    _ = ws!(recv guest_ws, "project_files", [ get "files", as array, dbg, expect empty ] );
}
