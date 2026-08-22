use dagr_cli::resolve_workspace;
use std::path::{Path, PathBuf};

fn p(s: &str) -> PathBuf {
    Path::new(s).to_path_buf()
}

#[test]
fn explicit_flag_wins_over_env_and_cwd() {
    assert_eq!(
        resolve_workspace(Some(p("/repo")), Some("/env".into()), p("/cwd")),
        p("/repo")
    );
}

#[test]
fn env_is_used_when_no_flag() {
    assert_eq!(
        resolve_workspace(None, Some("/env".into()), p("/cwd")),
        p("/env")
    );
}

#[test]
fn cwd_is_final_fallback() {
    assert_eq!(resolve_workspace(None, None, p("/cwd")), p("/cwd"));
}
