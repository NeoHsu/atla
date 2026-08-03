use super::*;

pub(super) async fn run(action: IssueAction, global: &Invocation) -> anyhow::Result<()> {
    let IssueAction::Transition { key, to, fields } = action else {
        unreachable!("issue action dispatcher passed the wrong variant to transition")
    };
    let ctx = AppContext::load(global)?;
    let profile_name = ctx.profile_name();
    let profile = ctx.profile();
    let fields = parse_fields(&fields)?;

    if global.dry_run {
        let url = format!(
            "{}/rest/api/3/issue/{}/transitions",
            profile.jira_api_base_url(),
            key
        );
        if let Some(to) = &to {
            println!(
                "Would GET {url}?expand=transitions.fields, then POST transition `{to}` using profile `{profile_name}`"
            );
        } else {
            println!("Would GET {url}?expand=transitions.fields using profile `{profile_name}`");
        }
        return Ok(());
    }

    let client = ctx.jira_client()?;
    if let Some(to) = to {
        let transition = client
            .transition_issue(&key, &to, fields)
            .await
            .with_context(|| {
                format!(
                    "failed to transition Jira issue `{key}` from {}",
                    client.instance_url()
                )
            })?;
        print_transition_update(&key, &transition, global)?;
    } else {
        let transitions = client.list_transitions(&key).await.with_context(|| {
            format!(
                "failed to list transitions for Jira issue `{key}` from {}",
                client.instance_url()
            )
        })?;
        if can_prompt(global) && !transitions.is_empty() {
            let selected = select_transition(&transitions)?;
            let transition_id = selected
                .id
                .as_deref()
                .or(selected.name.as_deref())
                .ok_or_else(|| {
                    anyhow::anyhow!("selected transition did not include an id or name")
                })?;
            let transition = client
                .transition_issue(&key, transition_id, fields)
                .await
                .with_context(|| {
                    format!(
                        "failed to transition Jira issue `{key}` from {}",
                        client.instance_url()
                    )
                })?;
            print_transition_update(&key, &transition, global)?;
        } else if can_prompt(global) {
            anyhow::bail!("no transitions available for issue `{key}`");
        } else {
            let names: Vec<_> = transitions
                .iter()
                .filter_map(|t| t.name.as_deref())
                .collect();
            anyhow::bail!(
                "--to is required in non-interactive mode; available transitions: {}",
                names.join(", ")
            );
        }
    }
    Ok(())
}
