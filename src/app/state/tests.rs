use crate::app::ui::{FindingKind, HostMapping, IdMapEntry};

use super::State;

#[test]
fn test_duplicate_username_not_allowed_in_subid() {
    let mut state = State {
        host_mapping: HostMapping {
            subuid: Vec::new(),
            subgid: Vec::new(),
        },
        ..State::default()
    };

    state.evaluate_findings();

    assert!(state.findings.is_empty());

    state.host_mapping.subuid = vec![
        IdMapEntry {
            host_user_id: "1000".into(),
            host_sub_id: 10000,
            host_sub_id_count: 65000,
        },
        IdMapEntry {
            host_user_id: "1000".into(),
            host_sub_id: 10000,
            host_sub_id_count: 65000,
        },
    ];

    state.evaluate_findings();

    assert_eq!(state.findings.len(), 1);
    assert_eq!(state.findings[0].kind, FindingKind::Bad);
    assert_eq!(
        state.findings[0].message,
        "Cannot have multiple entries for the same user"
    );
    assert_eq!(state.findings[0].host_mapping_highlights, vec![0, 1]);
    assert_eq!(state.findings[0].lxc_config_mapping_highlights, Vec::new());

    state.host_mapping.subgid = state.host_mapping.subuid;
    state.host_mapping.subuid = Vec::new();

    state.evaluate_findings();

    assert_eq!(state.findings.len(), 1);
    assert_eq!(state.findings[0].kind, FindingKind::Bad);
    assert_eq!(
        state.findings[0].message,
        "Cannot have multiple entries for the same group"
    );
    assert_eq!(state.findings[0].host_mapping_highlights, vec![0, 1]);
    assert_eq!(state.findings[0].lxc_config_mapping_highlights, Vec::new());
}
