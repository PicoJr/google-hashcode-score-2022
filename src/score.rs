use crate::data::{Id, PInput, POutput};
use ahash::AHashMap;
use anyhow::bail;
use log::debug;
use std::cmp::max;

pub(crate) type Score = usize;
pub(crate) type Time = usize;
pub(crate) type Level = usize;

#[derive(Debug)]
pub struct Contributor {
    id: Id,
    name: String,
    skills: Vec<Id>,
    next_availability: Time,
}

#[derive(Debug)]
pub struct Project {
    id: Id,
    name: String,
    skills: Vec<(Id, Level)>,
    days_to_completion: usize,
    score: usize,
    best_before: usize,
}

#[derive(Debug)]
pub struct PlannedProject {
    id: Id,
    contributors: Vec<Id>,
}

#[derive(Debug)]
pub struct PreComputed {
    contributors_id: AHashMap<String, Id>,
    projects_id: AHashMap<String, Id>,
    skills_id: AHashMap<String, Id>,
    levels: AHashMap<(Id, Id), Level>, // contributor id, skill id, level
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
            let current_skill_id = if let Some(existing_id) = skills_id.get(&*skill.name) {
                *existing_id
            } else {
                skills_id.insert(skill.name.clone(), skill_id);
                skill_id += 1;
                skill_id - 1
            };
            skills.push((current_skill_id, skill.level));
            debug!(
                "project {} ({}) require level {} in {} ({})",
                project.id, project.name, skill.level, current_skill_id, skill.name
            );
        }
        projects.push(Project {
            id: project.id,
            name: project.name.clone(),
            skills,
            days_to_completion: project.days_to_completion,
            score: project.score,
            best_before: project.best_before,
        })
    }
    let mut contributors_id: AHashMap<String, Id> = AHashMap::new();
    let mut contributors = vec![];
    let mut levels: AHashMap<(Id, Id), Level> = AHashMap::new();
    for contributor in &input.contributors {
        contributors_id.insert(contributor.name.clone(), contributor.id);
        let mut skills: Vec<Id> = vec![];
        for skill in &contributor.skills {
            if let Some(contributor_skill_id) = skills_id.get(&*skill.name) {
                skills.push(*contributor_skill_id);
                levels.insert((contributor.id, *contributor_skill_id), skill.level);
                debug!("levels: {:?}", levels);
                debug!(
                    "contributor {} ({}) level in {} ({}) is {}",
                    contributor.id, contributor.name, contributor_skill_id, skill.name, skill.level
                )
            } else {
                debug!(
                    "contributor {} has this skill: {} but it is useless",
                    contributor.id, skill.name
                );
                skills_id.insert(skill.name.clone(), skill_id);
                skill_id += 1;
            }
        }
        let skills = contributor
            .skills
            .iter()
            .map(|skill| skills_id.get(&*skill.name).cloned().unwrap_or(0))
            .collect();
        contributors.push(Contributor {
            id: contributor.id,
            name: contributor.name.clone(),
            skills,
            next_availability: 0, // ready to work on project at t = 0
        })
    }
    (
        PreComputed {
            contributors_id,
            projects_id,
            skills_id,
            levels,
        },
        contributors,
        projects,
    )
}

pub fn precompute_from_output(precomputed: &PreComputed, output: &POutput) -> Vec<PlannedProject> {
    let mut planned_projects: Vec<PlannedProject> = vec![];
    for project in &output.projects {
        let mut contributors = vec![];
        for role in &project.roles {
            let role_id = precomputed
                .contributors_id
                .get(&*role)
                .cloned()
                .unwrap_or(0);
            contributors.push(role_id);
        }
        planned_projects.push(PlannedProject {
            id: precomputed
                .projects_id
                .get(&*project.name)
                .cloned()
                .unwrap_or(0),
            contributors,
        })
    }
    planned_projects
}

pub fn compute_score(input: &PInput, output: &POutput) -> anyhow::Result<Score> {
    let (mut precomputed, mut contributors, projects) = precompute_from_input(input);
    let planned_projects = precompute_from_output(&precomputed, output);

    debug!("{:?}", precomputed);
    debug!("{:?}", contributors);
    debug!("{:?}", projects);
    debug!("{:?}", planned_projects);

    let mut score: Score = 0;

    for planned_project in &planned_projects {
        if let Some(project) = projects.get(planned_project.id) {
            let mut project_contributors: Vec<&Contributor> = vec![];
            for contributor_id in &planned_project.contributors {
                if let Some(c) = contributors.get(*contributor_id) {
                    project_contributors.push(c);
                }
            }

            debug!(
                "contributors for project {:?} are {:?}",
                project, project_contributors
            );

            for ((skill_id, level_required), contributor_for_this_role_id) in
                project.skills.iter().zip(&planned_project.contributors)
            {
                let max_level_among_contributors = project_contributors
                    .iter()
                    .map(|c| {
                        precomputed
                            .levels
                            .get(&(c.id, *skill_id))
                            .cloned()
                            .unwrap_or(0)
                    })
                    .max()
                    .unwrap_or(0);
                let mentoring_available = max_level_among_contributors >= *level_required;
                let level_required_with_mentoring: Level = if mentoring_available {
                    *level_required - 1
                } else {
                    *level_required
                };
                let contributor_level_for_this_role = precomputed
                    .levels
                    .get(&(*contributor_for_this_role_id, *skill_id))
                    .cloned()
                    .unwrap_or(0);
                if level_required_with_mentoring > contributor_level_for_this_role {
                    bail!(
                        "contributor {} level in {} is {} vs {} required for project {} ({}) (mentoring: {})",
                        contributor_for_this_role_id,
                        skill_id,
                        contributor_level_for_this_role,
                        level_required_with_mentoring,
                        project.id,
                        project.name,
                        mentoring_available
                    )
                }
            }

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
                    max(0, project.score as i64 - days_late as i64) as usize // days late > 0
                };

                debug!(
                    "just finished {} (start {}, end {}, late {}, score {})",
                    project.id, project_start_time, project_end_time, days_late, score_increment
                );

                score += score_increment;

                for contributor_id in &planned_project.contributors {
                    if let Some(c) = contributors.get_mut(*contributor_id) {
                        c.next_availability = project_end_time;
                    }
                }
            } else {
                bail!("could not compute project start time");
            }

            for ((skill_id, level_required), contributor_for_this_role_id) in
                project.skills.iter().zip(&planned_project.contributors)
            {
                if let Some(contributor_level) = precomputed
                    .levels
                    .get_mut(&(*contributor_for_this_role_id, *skill_id))
                {
                    if *contributor_level <= *level_required {
                        *contributor_level += 1;
                    }
                } else {
                    precomputed
                        .levels
                        .insert((*contributor_for_this_role_id, *skill_id), 1);
                }
            }
        } else {
            bail!("unknown project {}", planned_project.id);
        }
    }
    Ok(score)
}
