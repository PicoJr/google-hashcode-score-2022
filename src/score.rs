use crate::data::{Id, PInput, POutput};
use anyhow::bail;
use fxhash::FxHashMap;
use log::debug;
use std::cmp::max;

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

pub(crate) type Score = usize;
pub(crate) type Time = usize;
pub(crate) type Level = usize;
pub(crate) type LevelMap = FxHashMap<(Id, Id), Level>; // contributor id, skill id, level

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Contributor {
    id: Id,
    name: String,
    skills: Vec<Id>,
    next_availability: Time,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Project {
    id: Id,
    name: String,
    skills: Vec<(Id, Level)>,
    days_to_completion: usize,
    score: usize,
    best_before: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct PreComputed {
    contributors_id: FxHashMap<String, Id>,
    projects_id: FxHashMap<String, Id>,
    skills_id: FxHashMap<String, Id>,
    levels: LevelMap,
    contributors: Vec<Contributor>,
    projects: Vec<Project>,
}

#[derive(Debug)]
pub(crate) struct PlannedProject {
    id: Id,
    contributors: Vec<Id>,
}

pub(crate) fn precompute_from_input(input: &PInput) -> PreComputed {
    let mut projects_id: FxHashMap<String, Id> = FxHashMap::default();
    let mut projects: Vec<Project> = Vec::with_capacity(input.projects.len());
    let mut skills_id: FxHashMap<String, Id> = FxHashMap::default();
    let mut skill_id: Id = 0;
    for project in &input.projects {
        projects_id.insert(project.name.clone(), project.id);
        let mut skills = Vec::with_capacity(project.skills.len());
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
    let mut contributors_id: FxHashMap<String, Id> = FxHashMap::default();
    let mut contributors = Vec::with_capacity(input.contributors.len());
    let mut levels: FxHashMap<(Id, Id), Level> = FxHashMap::default();
    for contributor in &input.contributors {
        contributors_id.insert(contributor.name.clone(), contributor.id);
        let mut skills: Vec<Id> = Vec::with_capacity(contributor.skills.len());
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
        contributors.push(Contributor {
            id: contributor.id,
            name: contributor.name.clone(),
            skills,
            next_availability: 0, // ready to work on project at t = 0
        })
    }
    PreComputed {
        contributors_id,
        projects_id,
        skills_id,
        levels,
        contributors,
        projects,
    }
}

fn precompute_from_output(
    precomputed: &PreComputed,
    output: &POutput,
) -> anyhow::Result<Vec<PlannedProject>> {
    let mut planned_projects: Vec<PlannedProject> = Vec::with_capacity(output.projects.len());
    for project in &output.projects {
        let mut contributors = Vec::with_capacity(project.contributor_names.len());
        for contributor_name in &project.contributor_names {
            if let Some(contributor_id) = precomputed.contributors_id.get(&*contributor_name) {
                contributors.push(*contributor_id);
            } else {
                bail!(
                    "unknown contributor {} for project {}",
                    contributor_name,
                    project.name
                );
            }
        }
        if let Some(project_id) = precomputed.projects_id.get(&*project.name) {
            planned_projects.push(PlannedProject {
                id: *project_id,
                contributors,
            })
        } else {
            bail!("unknown project {}", project.name);
        }
    }
    Ok(planned_projects)
}

fn check_contributors_level(
    project: &Project,
    planned_project: &PlannedProject,
    project_contributors: &[&Contributor],
    levels_map: &LevelMap,
) -> anyhow::Result<()> {
    for ((skill_id, level_required), contributor_for_this_role_id) in
        project.skills.iter().zip(&planned_project.contributors)
    {
        let max_level_among_contributors = project_contributors
            .iter()
            .map(|c| levels_map.get(&(c.id, *skill_id)).cloned().unwrap_or(0))
            .max()
            .unwrap_or(0);
        let mentoring_available = max_level_among_contributors >= *level_required;
        let level_required_with_mentoring: Level = if mentoring_available {
            *level_required - 1
        } else {
            *level_required
        };
        let contributor_level_for_this_role = levels_map
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
    Ok(())
}

fn project_score(project_start_time: Time, project: &Project) -> (Score, Time) {
    let project_end_time: Time = project_start_time + project.days_to_completion;
    let days_late: i64 = project_end_time as i64 - project.best_before as i64;
    let score_increment = if days_late <= 0 {
        project.score
    } else {
        // late
        max(0, project.score as i64 - days_late as i64) as usize // days late > 0
    };

    debug!(
        "project {}: (start = {}, end = {}, late = {}, score = {})",
        project.id, project_start_time, project_end_time, days_late, score_increment
    );
    (score_increment, project_end_time)
}

fn update_next_availability(
    project_end_time: Time,
    planned_project: &PlannedProject,
    contributors: &mut [Contributor],
) {
    for contributor_id in &planned_project.contributors {
        if let Some(c) = contributors.get_mut(*contributor_id) {
            c.next_availability = project_end_time;
        }
    }
}

fn update_level(project: &Project, planned_project: &PlannedProject, levels_map: &mut LevelMap) {
    for ((skill_id, level_required), contributor_for_this_role_id) in
        project.skills.iter().zip(&planned_project.contributors)
    {
        if let Some(contributor_level) =
            levels_map.get_mut(&(*contributor_for_this_role_id, *skill_id))
        {
            if *contributor_level <= *level_required {
                *contributor_level += 1;
                debug!(
                    "contributor {} reached level {} in {}",
                    contributor_for_this_role_id, contributor_level, skill_id
                );
            }
        } else {
            levels_map.insert((*contributor_for_this_role_id, *skill_id), 1);
            debug!(
                "contributor {} reached level {} in {}",
                contributor_for_this_role_id, 1, skill_id
            );
        }
    }
}

pub(crate) fn decode_precomputed(bin_path: &Path) -> anyhow::Result<PreComputed> {
    let file = File::open(bin_path)?;
    let reader = BufReader::new(file);
    let decoded: PreComputed = bincode::deserialize_from(reader)?;
    Ok(decoded)
}

pub(crate) fn encode_precomputed(precomputed: &PreComputed, bin_path: &Path) -> anyhow::Result<()> {
    let encoded: Vec<u8> = bincode::serialize(&precomputed)?;
    let mut output = File::create(bin_path)?;
    output.write_all(encoded.as_slice())?;
    Ok(())
}

#[allow(unused)]
fn compute_score(input: &PInput, output: &POutput, disable_checks: bool) -> anyhow::Result<Score> {
    let mut precomputed = precompute_from_input(input);
    debug!("{:?}", precomputed);

    compute_score_precomputed(&mut precomputed, output, disable_checks)
}

pub(crate) fn compute_score_precomputed(
    precomputed: &mut PreComputed,
    output: &POutput,
    disable_checks: bool,
) -> anyhow::Result<Score> {
    let planned_projects = precompute_from_output(precomputed, output)?;
    let mut contributors = &mut precomputed.contributors;
    let projects = &precomputed.projects;

    debug!("{:?}", contributors);
    debug!("{:?}", projects);
    debug!("{:?}", planned_projects);

    let mut score: Score = 0;

    for planned_project in &planned_projects {
        if let Some(project) = projects.get(planned_project.id) {
            let mut project_contributors: Vec<&Contributor> =
                Vec::with_capacity(planned_project.contributors.len());
            for contributor_id in &planned_project.contributors {
                if let Some(c) = contributors.get(*contributor_id) {
                    project_contributors.push(c);
                }
            }

            debug!(
                "contributors for project {:?} are {:?}",
                project, project_contributors
            );

            if !disable_checks {
                check_contributors_level(
                    project,
                    planned_project,
                    &project_contributors,
                    &precomputed.levels,
                )?;
            }

            if let Some(project_start_time) = project_contributors
                .iter()
                .map(|c| c.next_availability)
                .max()
            {
                let (score_increment, project_end_time) =
                    project_score(project_start_time, project);
                score += score_increment;
                update_next_availability(project_end_time, planned_project, &mut contributors);
            } else {
                bail!("could not compute project start time");
            }

            // update contributors level
            update_level(project, planned_project, &mut precomputed.levels);
        } else {
            bail!("unknown project {}", planned_project.id);
        }
    }
    Ok(score)
}
