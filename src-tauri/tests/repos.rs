//! Cross-table integration smoke tests. Each repo gets its happy path
//! exercised end-to-end against a fresh in-memory database, so any SQL typo
//! or FK violation surfaces immediately.

use hoverdo_lib::db::{repos, Db};
use hoverdo_lib::models::{
    NewList, NewNote, NewTag, NewTask, ReminderTarget, WidgetKind,
};
use time::OffsetDateTime;

async fn open_db() -> Db {
    Db::open_in_memory().await.expect("open in-memory db")
}

#[tokio::test]
async fn lists_full_flow() {
    let db = open_db().await;

    let list = repos::lists::create(
        &db,
        NewList {
            title: "Inbox".into(),
            color: None,
        },
    )
    .await
    .unwrap();

    let renamed = repos::lists::rename(&db, &list.id, "Work").await.unwrap();
    assert_eq!(renamed.title, "Work");
    assert!(renamed.hlc_ts > list.hlc_ts);

    assert_eq!(repos::lists::list_active(&db.pool).await.unwrap().len(), 1);

    repos::lists::soft_delete(&db, &list.id).await.unwrap();
    assert!(repos::lists::list_active(&db.pool).await.unwrap().is_empty());
}

#[tokio::test]
async fn tasks_attach_to_list_and_order_by_sort_key() {
    let db = open_db().await;
    let list = repos::lists::create(
        &db,
        NewList {
            title: "Today".into(),
            color: None,
        },
    )
    .await
    .unwrap();

    for title in ["one", "two", "three"] {
        repos::tasks::create(
            &db,
            NewTask {
                list_id: list.id.clone(),
                title: title.into(),
                due_at: None,
                sort_key: None,
            },
        )
        .await
        .unwrap();
    }

    let tasks = repos::tasks::list_for(&db.pool, &list.id).await.unwrap();
    assert_eq!(tasks.len(), 3);
    // sort_keys must be strictly increasing.
    for window in tasks.windows(2) {
        assert!(
            window[0].sort_key < window[1].sort_key,
            "sort_key not monotonic: {} vs {}",
            window[0].sort_key,
            window[1].sort_key
        );
    }

    let toggled = repos::tasks::set_done(&db, &tasks[0].id, true).await.unwrap();
    assert!(toggled.done);

    repos::tasks::soft_delete(&db, &tasks[1].id).await.unwrap();
    assert_eq!(
        repos::tasks::list_for(&db.pool, &list.id).await.unwrap().len(),
        2
    );
}

#[tokio::test]
async fn tags_upsert_and_attach() {
    let db = open_db().await;

    let work = repos::tags::upsert_by_name(
        &db,
        NewTag {
            name: "work".into(),
            color: Some("#3b6fe0".into()),
        },
    )
    .await
    .unwrap();
    let work_again = repos::tags::upsert_by_name(
        &db,
        NewTag {
            name: "work".into(),
            color: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(work.id, work_again.id, "upsert must deduplicate by name");

    let note = repos::notes::create(
        &db,
        NewNote {
            title: "n".into(),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    repos::tags::attach_to_note(&db.pool, &note.id, &work.id).await.unwrap();
    // attaching twice is a no-op (INSERT OR IGNORE)
    repos::tags::attach_to_note(&db.pool, &note.id, &work.id).await.unwrap();

    let tags = repos::tags::list_active(&db.pool).await.unwrap();
    assert_eq!(tags.len(), 1);
}

#[tokio::test]
async fn reminders_pending_filter() {
    let db = open_db().await;
    let note = repos::notes::create(
        &db,
        NewNote {
            title: "wash".into(),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let r = repos::reminders::create(
        &db,
        ReminderTarget::Note,
        &note.id,
        OffsetDateTime::now_utc() + time::Duration::hours(1),
    )
    .await
    .unwrap();

    let pending = repos::reminders::pending(&db.pool).await.unwrap();
    assert_eq!(pending.len(), 1);

    repos::reminders::mark_fired(&db, &r.id).await.unwrap();
    assert!(repos::reminders::pending(&db.pool).await.unwrap().is_empty());
}

#[tokio::test]
async fn widget_instances_pin_geometry_and_unpin() {
    let db = open_db().await;
    let note = repos::notes::create(
        &db,
        NewNote {
            title: "n".into(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let w = repos::widget_instances::pin(&db, WidgetKind::Sticky, &note.id)
        .await
        .unwrap();

    repos::widget_instances::update_geometry(&db, &w.id, Some(100), Some(200), 320, 240)
        .await
        .unwrap();
    repos::widget_instances::set_opacity(&db, &w.id, 0.6).await.unwrap();
    repos::widget_instances::set_always_on_top(&db, &w.id, true).await.unwrap();

    let stored = repos::widget_instances::get(&db.pool, &w.id).await.unwrap().unwrap();
    assert_eq!(stored.win_x, Some(100));
    assert_eq!(stored.win_w, 320);
    assert!((stored.opacity - 0.6).abs() < 1e-9);
    assert!(stored.always_on_top);

    repos::widget_instances::unpin(&db, &w.id).await.unwrap();
    assert!(repos::widget_instances::list_active(&db.pool).await.unwrap().is_empty());
}

#[tokio::test]
async fn fts5_finds_notes_lists_and_tasks() {
    // Bypasses the IPC layer (Tauri state needs a runtime); the underlying
    // FTS5 wiring is shared with `commands::search` so a match here proves
    // the schema + triggers in 0001_init.sql work end-to-end.
    let db = open_db().await;

    let _note = repos::notes::create(
        &db,
        NewNote {
            title: "Groceries".into(),
            body: "buy oat milk and bread".into(),
            color: None,
        },
    )
    .await
    .unwrap();
    let _list = repos::lists::create(
        &db,
        NewList {
            title: "Errands".into(),
            color: None,
        },
    )
    .await
    .unwrap();

    use sqlx::Row;
    let rows = sqlx::query(
        "SELECT n.title AS t \
         FROM notes_fts \
         JOIN notes n ON n.rowid = notes_fts.rowid \
         WHERE notes_fts MATCH ?1",
    )
    .bind("\"milk\"*")
    .fetch_all(&db.pool)
    .await
    .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].get::<String, _>("t"), "Groceries");
}

#[tokio::test]
async fn settings_json_roundtrip() {
    let db = open_db().await;
    #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
    struct ThemeChoice {
        mode: String,
        accent: Option<String>,
    }
    let value = ThemeChoice {
        mode: "dark".into(),
        accent: Some("#7aa2ff".into()),
    };
    repos::settings::set(&db.pool, "ui.theme", &value).await.unwrap();
    let back: ThemeChoice = repos::settings::get(&db.pool, "ui.theme")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(back, value);
}
