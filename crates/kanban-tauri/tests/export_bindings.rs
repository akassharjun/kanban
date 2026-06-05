//! Generates the TypeScript bindings. Run with `cargo test -p kanban-tauri --test export_bindings`.
//! The output `ui/src/data/bindings.ts` is git-tracked.
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

#[test]
fn export_typescript_bindings() {
    use specta_typescript::{BigIntExportBehavior, Typescript};
    // `i64` DTO fields (`ApplyResult.op_id`, `IssueDto.seq`, `StatusDto.position`)
    // would error under the default `BigIntExportBehavior::Fail`. SQLite ids/seqs
    // fit safely under 2^53, so we emit them as TS `number`.
    // tauri-specta always emits the `TAURI_CHANNEL` import and `__makeEvents__`
    // helper, which are unused when there are no events/channels. The UI's
    // tsconfig sets `noUnusedLocals`, so exempt this generated file from tsc.
    kanban_tauri::specta_builder()
        .export(
            Typescript::default()
                .bigint(BigIntExportBehavior::Number)
                .header("// @ts-nocheck\n"),
            "../../ui/src/data/bindings.ts",
        )
        .expect("failed to export TypeScript bindings");
}
