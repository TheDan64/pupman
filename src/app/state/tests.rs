use std::str::FromStr;

use crate::{
    app::ui::{FindingKind, HostMapping, IdMapEntry},
    lxc::Config,
    metadata::Metadata,
};

use super::State;

#[test]
fn test_duplicate_username_not_allowed_in_subid() {
    let pve_md = Metadata {
        is_pve: true,
        ..Metadata::default()
    };
    let mut state = State {
        host_mapping: HostMapping {
            subuid: Vec::new(),
            subgid: Vec::new(),
        },
        ..State::default()
    };

    state.evaluate_findings(&pve_md);

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

    state.evaluate_findings(&pve_md);

    assert_eq!(state.findings.len(), 1);
    assert_eq!(state.findings[0].kind, FindingKind::Bad);
    assert_eq!(
        state.findings[0].message,
        "[PVE] Cannot have multiple entries for the same user"
    );
    assert_eq!(state.findings[0].host_mapping_highlights, vec![0, 1]);
    assert_eq!(state.findings[0].lxc_config_mapping_highlights, Vec::new());

    state.host_mapping.subgid = state.host_mapping.subuid;
    state.host_mapping.subuid = Vec::new();

    state.evaluate_findings(&pve_md);

    assert_eq!(state.findings.len(), 1);
    assert_eq!(state.findings[0].kind, FindingKind::Bad);
    assert_eq!(
        state.findings[0].message,
        "[PVE] Cannot have multiple entries for the same group"
    );
    assert_eq!(state.findings[0].host_mapping_highlights, vec![0, 1]);
    assert_eq!(state.findings[0].lxc_config_mapping_highlights, Vec::new());
}

#[test]
fn test_subid_out_of_range() {
    let config = r#"
lxc.idmap = u 1000 10000 65000
lxc.idmap = g 1000 10000 65000
"#;
    let config2 = r#"
lxc.idmap = u 1000 10000 65001
lxc.idmap = g 1000 10000 65001
"#;
    let mut state = State {
        host_mapping: HostMapping {
            subuid: vec![IdMapEntry {
                host_user_id: "1000".into(),
                host_sub_id: 10000,
                host_sub_id_count: 65000,
            }],
            subgid: vec![IdMapEntry {
                host_user_id: "1000".into(),
                host_sub_id: 10000,
                host_sub_id_count: 65000,
            }],
        },
        lxc_configs: [("test.conf".into(), Config::from_str(config).unwrap())]
            .into_iter()
            .collect(),
        ..State::default()
    };

    state.evaluate_findings(&Metadata::default());

    assert!(state.findings.is_empty());

    state.lxc_configs = [("test.conf".into(), Config::from_str(config2).unwrap())]
        .into_iter()
        .collect();

    state.evaluate_findings(&Metadata::default());

    assert_eq!(state.findings.len(), 2);
    assert_eq!(state.findings[0].kind, FindingKind::Bad);
    assert_eq!(
        state.findings[0].message,
        "LXC config's host sub uid range outside of host mapping range"
    );
    assert_eq!(state.findings[0].host_mapping_highlights, [0]);
    assert_eq!(state.findings[0].lxc_config_mapping_highlights, [0]);
    assert_eq!(state.findings[1].kind, FindingKind::Bad);
    assert_eq!(
        state.findings[1].message,
        "LXC config's host sub gid range outside of host mapping range"
    );
    assert_eq!(state.findings[1].host_mapping_highlights, [1]);
    assert_eq!(state.findings[1].lxc_config_mapping_highlights, [1]);
}
