use crate::models::{Group, Company, MatchResult};
use std::collections::{HashMap, VecDeque};

pub fn stable_matching(groups: &[Group], companies: &[Company]) -> Vec<MatchResult> {
    if groups.is_empty() || companies.is_empty() {
        println!("No groups or companies to match");
        return vec![];
    }

    let _group_idx: HashMap<String, usize> = groups.iter()
        .enumerate()
        .map(|(i, g)| (g.name.clone(), i))
        .collect();

    let company_idx: HashMap<String, usize> = companies.iter()
        .enumerate()
        .map(|(i, c)| (c.name.clone(), i))
        .collect();

    // Normalny Gale-Shapley z preferencjami
    let mut free_groups: VecDeque<usize> = (0..groups.len()).collect();
    let mut next_proposal: Vec<usize> = vec![0; groups.len()];
    let mut company_partner: Vec<Option<usize>> = vec![None; companies.len()];
    let mut company_score: Vec<HashMap<String, i32>> = vec![HashMap::new(); companies.len()];
    
    for (i, c) in companies.iter().enumerate() {
        for (position, group_name) in c.preferences.iter().enumerate() {
            let base_score = (position as i32 + 1) * 10;
            let mut extra_points = 0;

            if position < 3 {
                extra_points -= 5;
            }
            if c.preferences.len() <= 2 {
                extra_points -= 3;
            }
            
            let final_score = base_score + extra_points;
            company_score[i].insert(group_name.clone(), final_score);
        }
    }

    // Normalny Gale-Shapley
    let mut matched_groups = vec![false; groups.len()];
    
    while let Some(g_idx) = free_groups.pop_front() {
        let group = &groups[g_idx];
        if next_proposal[g_idx] >= group.preferences.len() {
            continue;
        }

        let company_name = &group.preferences[next_proposal[g_idx]];
        next_proposal[g_idx] += 1;

        let Some(&c_idx) = company_idx.get(company_name) else {
            free_groups.push_back(g_idx);
            continue;
        };

        match company_partner[c_idx] {
            None => {
                if company_score[c_idx].contains_key(&group.name) {
                    company_partner[c_idx] = Some(g_idx);
                    matched_groups[g_idx] = true;
                } else {
                    free_groups.push_back(g_idx);
                }
            }
            Some(current_g_idx) => {
                let score_new = company_score[c_idx].get(&group.name).copied().unwrap_or(i32::MAX);
                let score_old = company_score[c_idx].get(&groups[current_g_idx].name).copied().unwrap_or(i32::MAX);
                
                if score_new < score_old {
                    company_partner[c_idx] = Some(g_idx);
                    matched_groups[g_idx] = true;
                    matched_groups[current_g_idx] = false;
                    free_groups.push_back(current_g_idx);
                } else {
                    free_groups.push_back(g_idx);
                }
            }
        }
    }
    
    
    // Wolne grupy
    let unmatched_groups: Vec<usize> = matched_groups.iter()
        .enumerate()
        .filter(|(_, matched)| !**matched)
        .map(|(idx, _)| idx)
        .collect();
    
    // Wolne firmy
    let free_companies: Vec<usize> = company_partner.iter() 
        .enumerate()
        .filter(|(_, partner)| partner.is_none())
        .map(|(idx, _)| idx)
        .collect();

    // Jeśli nie ma wystarczająco wolnych firm, niektóre firmy dostaną 2 grupy
    let mut extra_assignments = vec![];
    
    for (i, &g_idx) in unmatched_groups.iter().enumerate() {
        let group = &groups[g_idx];
        
        if i < free_companies.len() {
            let c_idx = free_companies[i];
            company_partner[c_idx] = Some(g_idx);
            matched_groups[g_idx] = true;
        } else {
            let mut best_company = None;
            let mut best_score = i32::MAX;
            
            for c_idx in 0..companies.len() {
                if let Some(score) = company_score[c_idx].get(&group.name) {
                    if *score < best_score {
                        best_score = *score;
                        best_company = Some(c_idx);
                    }
                }
            }
            
            if let Some(c_idx) = best_company {
                extra_assignments.push((group.name.clone(), companies[c_idx].name.clone()));
            } else {
                let c_idx = 0;
                extra_assignments.push((group.name.clone(), companies[c_idx].name.clone()));
            }
        }
    }

    
    let mut results = Vec::new();
    
    for (c_idx, g_idx_opt) in company_partner.iter().enumerate() {
        if let Some(g_idx) = g_idx_opt {
            results.push(MatchResult {
                group: groups[*g_idx].name.clone(),
                company: companies[c_idx].name.clone(),
            });
        }
    }
    for (group_name, company_name) in extra_assignments {
        results.push(MatchResult {
            group: group_name,
            company: company_name,
        });
    }
    results.sort_by(|a, b| a.group.cmp(&b.group));
    results
}