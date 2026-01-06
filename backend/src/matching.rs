use crate::models::{Group, GroupPreferences, MatchResult, Project, ProjectPreferences};
use std::collections::{HashMap, VecDeque};

pub fn stable_matching(
    groups: &[Group],
    projects: &[Project],
    group_prefs: &[GroupPreferences],
    project_prefs: &[ProjectPreferences],
) -> Vec<MatchResult> {
    if groups.is_empty() || projects.is_empty() {
        return vec![];
    }
    let group_idx: HashMap<&str, usize> = groups
        .iter()
        .enumerate()
        .map(|(i, g)| (g.email.as_str(), i))
        .collect();

    let project_idx: HashMap<&str, usize> = projects
        .iter()
        .enumerate()
        .map(|(i, p)| (p.id.as_str(), i))
        .collect();
    let mut g_pref_list: Vec<Vec<String>> = vec![vec![]; groups.len()];
    for gp in group_prefs.iter() {
        if let Some(&gi) = group_idx.get(gp.group_email.as_str()) {
            g_pref_list[gi] = gp.project_ids_ranked.clone();
        }
    }

    let mut project_score: Vec<HashMap<&str, i32>> = vec![HashMap::new(); projects.len()];
    for (p_idx, _p) in projects.iter().enumerate() {
        let _ = p_idx;
    }
    for pp in project_prefs.iter() {
        let Some(&p_idx) = project_idx.get(pp.project_id.as_str()) else {
            continue;
        };

        for (position, group_email) in pp.group_emails_ranked.iter().enumerate() {
            let base_score = (position as i32 + 1) * 10;
            let mut extra_points = 0;

            if position < 3 {
                extra_points -= 5;
            }
            if pp.group_emails_ranked.len() <= 2 {
                extra_points -= 3;
            }

            let final_score = base_score + extra_points;
            project_score[p_idx].insert(group_email.as_str(), final_score);
        }
    }

    let mut free_groups: VecDeque<usize> = (0..groups.len()).collect();
    let mut next_proposal: Vec<usize> = vec![0; groups.len()];
    let mut project_partner: Vec<Option<usize>> = vec![None; projects.len()];

    while let Some(g_idx) = free_groups.pop_front() {
        if next_proposal[g_idx] >= g_pref_list[g_idx].len() {
            continue;
        }

        let project_id = &g_pref_list[g_idx][next_proposal[g_idx]];
        next_proposal[g_idx] += 1;

        let Some(&p_idx) = project_idx.get(project_id.as_str()) else {
            free_groups.push_back(g_idx);
            continue;
        };

        if !projects[p_idx].active || projects[p_idx].capacity == 0 {
            free_groups.push_back(g_idx);
            continue;
        }

        let group_email = groups[g_idx].email.as_str();
        if !project_score[p_idx].contains_key(group_email) {
            free_groups.push_back(g_idx);
            continue;
        }

        match project_partner[p_idx] {
            None => {
                project_partner[p_idx] = Some(g_idx);
            }
            Some(current_g_idx) => {
                let current_email = groups[current_g_idx].email.as_str();

                let score_new = project_score[p_idx]
                    .get(group_email)
                    .copied()
                    .unwrap_or(i32::MAX);

                let score_old = project_score[p_idx]
                    .get(current_email)
                    .copied()
                    .unwrap_or(i32::MAX);

                if score_new < score_old {
                    project_partner[p_idx] = Some(g_idx);
                    free_groups.push_back(current_g_idx);
                } else {
                    free_groups.push_back(g_idx);
                }
            }
        }
    }

    let mut results = Vec::new();
    for (p_idx, g_idx_opt) in project_partner.iter().enumerate() {
        if let Some(g_idx) = g_idx_opt {
            results.push(MatchResult {
                group_email: groups[*g_idx].email.clone(),
                project_id: projects[p_idx].id.clone(),
                project_name: projects[p_idx].name.clone(),
                company_email: projects[p_idx].company_email.clone(),
            });
        }
    }

    results.sort_by(|a, b| a.group_email.cmp(&b.group_email));
    results
}
