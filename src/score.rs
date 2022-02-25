use crate::data::{Id, PInput, POutput};
use ahash::AHashMap;
use anyhow::bail;
use std::cmp::max;

pub(crate) type Score = usize;
pub(crate) type Time = usize;

#[derive(Debug)]
pub struct Contributor {
    id: Id,
    skills: Vec<Id>,
    next_availability: Time,
}

#[derive(Debug)]
pub struct Project {
    id: Id,
    skills: Vec<Id>,
    days_to_completion: usize,
    score: usize,
    best_before: usize,
}

#[derive(Debug)]
pub struct PlannedProject {
    id: Id,
    roles: Vec<Id>,
}

#[derive(Debug)]
pub struct PreComputed {
    contributors_id: AHashMap<String, Id>,
    projects_id: AHashMap<String, Id>,
    skills_id: AHashMap<String, Id>,
}

pub fn precompute_from_input(input: &PInput) -> (PreComputed, Vec<Contributor>, Vec<Project>) {
    let mut projects_id: AHashMap<String, Id> = AHashMap::new();
    let mut projects: Vec<Project> = vec![];
    let mut skills_id: AHashMap<String, Id> = AHashMap::new();
    let mut skill_id: Id = 0;
    for project in &input.projects {
        projects_id.insert(project.name.clone(), project.id);
        let mut skills = vec![];
        for skill in &project.skills {
            if !skills_id.contains_key(&*skill.name) {
                skills_id.insert(skill.name.clone(), skill_id);
                skills.push(skill_id);
                skill_id += 1;
            }
        }
        projects.push(Project {
            id: project.id,
            skills,
            days_to_completion: project.days_to_completion,
            score: project.score,
            best_before: project.best_before,
        })
    }
    let mut contributors_id: AHashMap<String, Id> = AHashMap::new();
    let mut contributors = vec![];
    for contributor in &input.contributors {
        contributors_id.insert(contributor.name.clone(), contributor.id);
        let skills = contributor
            .skills
            .iter()
            .map(|skill| skills_id.get(&*skill.name).cloned().unwrap_or(0))
            .collect();
        contributors.push(Contributor {
            id: contributor.id,
            skills,
            next_availability: 0, // ready to work on project at t = 0
        })
    }
    (
        PreComputed {
            contributors_id,
            projects_id,
            skills_id,
        },
        contributors,
        projects,
    )
}

pub fn precompute_from_output(precomputed: &PreComputed, output: &POutput) -> Vec<PlannedProject> {
    let mut planned_projects: Vec<PlannedProject> = vec![];
    for project in &output.projects {
        let mut roles = vec![];
        for role in &project.roles {
            let role_id = precomputed
                .contributors_id
                .get(&*role)
                .cloned()
                .unwrap_or(0);
            roles.push(role_id);
        }
        planned_projects.push(PlannedProject {
            id: precomputed
                .projects_id
                .get(&*project.name)
                .cloned()
                .unwrap_or(0),
            roles,
        })
    }
    planned_projects
}

pub fn compute_score(input: &PInput, output: &POutput) -> anyhow::Result<Score> {
    let (precomputed, mut contributors, projects) = precompute_from_input(input);
    let planned_projects = precompute_from_output(&precomputed, output);

    println!("{:?}", precomputed);
    println!("{:?}", contributors);
    println!("{:?}", projects);
    println!("{:?}", planned_projects);

    let mut score: Score = 0;

    for planned_project in &planned_projects {
        if let Some(project) = projects.get(planned_project.id) {
            let mut project_contributors: Vec<&Contributor> = vec![];
            for contributor_id in &planned_project.roles {
                if let Some(c) = contributors.get(*contributor_id) {
                    project_contributors.push(c);
                }
            }
            // todo check contributors are skilled enough for this project
            if let Some(project_start_time) = project_contributors
                .iter()
                .map(|c| c.next_availability)
                .max()
            {
                let project_end_time: Time = project_start_time + project.days_to_completion;
                let days_late: i64 = project_end_time as i64 - project.best_before as i64;
                let score_increment = if days_late <= 0 {
                    project.score
                } else {
                    // late
                    max(0, project.score - days_late as usize) // days late > 0
                };

                println!(
                    "just finished {} (start {}, end {}, late {}, score {})",
                    project.id, project_start_time, project_end_time, days_late, score_increment
                );
                score += score_increment;

                for contributor_id in &planned_project.roles {
                    if let Some(c) = contributors.get_mut(*contributor_id) {
                        c.next_availability = project_end_time;
                    }
                }
            } else {
                bail!("could not compute project start time");
            }
        } else {
            bail!("unknown project {}", planned_project.id);
        }
    }
    Ok(score)
}
