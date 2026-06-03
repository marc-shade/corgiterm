use std::collections::HashMap;

use corgiterm_config::{parse_ssh_config, Snippet, SnippetsConfig};

#[test]
fn ssh_fixture_parses_hosts_options_and_commands() {
    let hosts = parse_ssh_config(
        include_str!("../../../tests/fixtures/ssh/config.sample"),
        22,
    );

    assert_eq!(hosts.len(), 3);

    let prod = hosts
        .iter()
        .find(|host| host.name == "prod-web")
        .expect("prod-web host should be parsed");
    assert_eq!(prod.hostname, "prod-web.example.com");
    assert_eq!(prod.username.as_deref(), Some("deploy"));
    assert_eq!(prod.port, 2222);
    assert_eq!(
        prod.identity_file
            .as_ref()
            .map(|path| path.display().to_string()),
        Some("~/.ssh/prod_web".to_string())
    );
    assert!(prod.options.contains(&"ProxyJump bastion".to_string()));
    assert!(prod.options.contains(&"ServerAliveInterval 30".to_string()));
    assert_eq!(
        prod.build_command(),
        vec![
            "ssh",
            "-p",
            "2222",
            "-i",
            "~/.ssh/prod_web",
            "ProxyJump bastion",
            "ServerAliveInterval 30",
            "deploy@prod-web.example.com",
        ]
    );

    let staging = hosts
        .iter()
        .find(|host| host.name == "staging-db")
        .expect("staging-db host should be parsed");
    assert_eq!(staging.port, 22, "default port should be applied");
    assert_eq!(staging.display_string(), "postgres@10.20.30.40:22");
}

#[test]
fn snippets_fixture_exercises_variables_search_and_sorting() {
    let config: SnippetsConfig = serde_json::from_str(include_str!(
        "../../../tests/fixtures/snippets/sample-snippets.json"
    ))
    .expect("sample snippets fixture should parse");

    assert_eq!(config.snippets.len(), 3);
    assert_eq!(config.top_categories(), vec!["Docker", "Git", "SSH"]);
    assert_eq!(
        config.tags(),
        vec!["branch", "container", "docker", "git", "remote", "ssh"]
    );

    let git = config
        .find("git-feature-branch")
        .expect("git fixture snippet should exist");
    assert!(git.pinned);
    assert_eq!(git.top_category(), Some("Git"));
    assert_eq!(git.category_parts(), vec!["Git", "Branches"]);

    let variables = git.extract_variables();
    assert_eq!(
        variables.len(),
        1,
        "duplicate variables should be collapsed"
    );
    assert_eq!(variables[0].name, "branch_name");
    assert_eq!(variables[0].hint.as_deref(), Some("short branch name"));

    let substituted = git.substitute_variables(&HashMap::from([(
        "branch_name".to_string(),
        "terminal-tests".to_string(),
    )]));
    assert_eq!(
        substituted,
        "git checkout -b feature/terminal-tests && git push -u origin feature/terminal-tests"
    );

    let ssh = config
        .find("ssh-connect")
        .expect("ssh snippet should exist");
    assert_eq!(
        ssh.substitute_variables(&HashMap::from([(
            "host".to_string(),
            "example.com".to_string(),
        )])),
        "ssh deploy@example.com -p 22",
        "defaults should fill user and port while provided host is substituted"
    );

    let docker = config.search("container");
    assert_eq!(docker.len(), 1);
    assert_eq!(docker[0].id, "docker-run-port");

    let by_usage: Vec<&str> = config
        .by_usage()
        .into_iter()
        .map(|s| s.id.as_str())
        .collect();
    assert_eq!(
        by_usage,
        vec!["git-feature-branch", "docker-run-port", "ssh-connect"]
    );

    let pinned: Vec<&Snippet> = config.pinned();
    assert_eq!(pinned.len(), 1);
    assert_eq!(pinned[0].id, "git-feature-branch");
}
