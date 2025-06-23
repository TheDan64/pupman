use std::str::FromStr;

use crate::app::ui::{FindingKind, HostMapping, IdMapEntry};
use crate::fs::subid::SubID;
use crate::lxc::config::Config;

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

    assert_eq!(state.findings.len(), 1);
    assert_eq!(state.findings[0].kind, FindingKind::Good);

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
    assert_eq!(
        state.findings[0].host_mapping_highlights,
        vec![("1000".into(), SubID::UID)]
    );
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
    assert_eq!(
        state.findings[0].host_mapping_highlights,
        vec![("1000".into(), SubID::GID)]
    );
    assert_eq!(state.findings[0].lxc_config_mapping_highlights, Vec::new());
}

#[test]
fn test_subid_out_of_range() -> color_eyre::Result<()> {
    let config = r#"
lxc.idmap = u 0 10000 65000
lxc.idmap = g 0 10000 65000
unprivileged: 1
"#;
    let config2 = r#"
lxc.idmap = u 0 10000 65001
lxc.idmap = g 0 10000 65001
unprivileged: 1
"#;
    let mut state = State {
        host_mapping: HostMapping {
            subuid: vec![IdMapEntry {
                host_user_id: "0".into(),
                host_sub_id: 10000,
                host_sub_id_count: 65000,
            }],
            subgid: vec![IdMapEntry {
                host_user_id: "0".into(),
                host_sub_id: 10000,
                host_sub_id_count: 65000,
            }],
        },
        lxc_configs: [("test.conf".into(), Config::from_str(config)?)].into_iter().collect(),
        ..State::default()
    };

    state.evaluate_findings();

    assert!(state.findings.iter().all(|f| f.kind == FindingKind::Good));

    state.lxc_configs = [("test.conf".into(), Config::from_str(config2)?)].into_iter().collect();

    state.evaluate_findings();

    let findings = state
        .findings
        .iter()
        .filter(|f| f.kind == FindingKind::Bad)
        .collect::<Vec<_>>();

    assert_eq!(findings.len(), 2);
    assert_eq!(findings[0].kind, FindingKind::Bad);
    assert_eq!(
        findings[0].message,
        "LXC config's host sub uid range outside of host mapping range"
    );
    assert_eq!(findings[0].host_mapping_highlights, [("0".into(), SubID::UID)]);
    assert_eq!(
        findings[0].lxc_config_mapping_highlights,
        [("test.conf".into(), SubID::UID)]
    );
    assert_eq!(findings[1].kind, FindingKind::Bad);
    assert_eq!(
        findings[1].message,
        "LXC config's host sub gid range outside of host mapping range"
    );
    assert_eq!(findings[1].host_mapping_highlights, [("0".into(), SubID::GID)]);
    assert_eq!(
        findings[1].lxc_config_mapping_highlights,
        [("test.conf".into(), SubID::GID)]
    );

    Ok(())
}
