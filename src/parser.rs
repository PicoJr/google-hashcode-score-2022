use anyhow::bail;
use nom::bytes::complete::{take_while1, take_while_m_n};
use nom::combinator::map_res;
use nom::error::{convert_error, VerboseError};
use nom::multi::{many_m_n, separated_list1};
use nom::sequence::{terminated, tuple};
use nom::IResult;

use crate::data::{PContributor, PContributorSkill, PInput, POutput, PPlannedProject, PProject};
use nom::character::complete::{digit1, line_ending};

pub(crate) type N = usize;
pub(crate) type Res<T, U> = IResult<T, U, VerboseError<T>>;

fn number(input: &str) -> Res<&str, &str> {
    digit1(input)
}

fn positive_number(input: &str) -> Res<&str, N> {
    map_res(number, |out| N::from_str_radix(out, 10))(input)
}

fn single_space(input: &str) -> Res<&str, &str> {
    take_while_m_n(1, 1, |c: char| c == ' ')(input)
}

fn non_space_or_unix_eol(input: &str) -> Res<&str, &str> {
    take_while1(|c: char| c != ' ' && c != '\n')(input)
}

fn contributor_skill(input: &str) -> Res<&str, PContributorSkill> {
    let (i, (skill_name, _, skill_level)) =
        tuple((non_space_or_unix_eol, single_space, positive_number))(input)?;
    Ok((
        i,
        PContributorSkill {
            name: skill_name.to_string(),
            level: skill_level,
        },
    ))
}

fn contributor(input: &str) -> Res<&str, PContributor> {
    let (i, (name, _, n_skills)) = terminated(
        tuple((non_space_or_unix_eol, single_space, positive_number)),
        line_ending,
    )(input)?;
    let (i, skills) = many_m_n(
        n_skills,
        n_skills,
        terminated(contributor_skill, line_ending),
    )(i)?;
    Ok((
        i,
        PContributor {
            name: name.to_string(),
            n_skills,
            skills,
            id: 0,
        },
    ))
}

fn project(input: &str) -> Res<&str, PProject> {
    let (i, (project_name, _)) = tuple((non_space_or_unix_eol, single_space))(input)?;
    let (i, (days_to_completion, _)) = tuple((positive_number, single_space))(i)?;
    let (i, (score, _)) = tuple((positive_number, single_space))(i)?;
    let (i, (best_before, _)) = tuple((positive_number, single_space))(i)?;
    let (i, n_roles) = terminated(positive_number, line_ending)(i)?;
    let (i, skills) = many_m_n(n_roles, n_roles, terminated(contributor_skill, line_ending))(i)?;
    Ok((
        i,
        PProject {
            name: project_name.to_string(),
            days_to_completion,
            score,
            best_before,
            n_roles,
            skills,
            id: 0,
        },
    ))
}

fn _parse_input(input: &str) -> Res<&str, PInput> {
    let (i, (n_contributors, _, n_projects)) = terminated(
        tuple((positive_number, single_space, positive_number)),
        line_ending,
    )(input)?;
    let (i, mut contributors) = many_m_n(n_contributors, n_contributors, contributor)(i)?;
    for (id, c) in contributors.iter_mut().enumerate() {
        c.id = id;
    }
    let (i, mut projects) = many_m_n(n_projects, n_projects, project)(i)?;
    for (id, mut p) in projects.iter_mut().enumerate() {
        p.id = id;
    }
    Ok((
        i,
        PInput {
            n_contributors,
            n_projects,
            contributors,
            projects,
        },
    ))
}

pub fn parse_input(s: &str) -> anyhow::Result<PInput> {
    match _parse_input(s) {
        Ok((_, data)) => Ok(data),
        Err(nom::Err::Error(err)) => bail!("{}", convert_error(s, err.clone())),
        _ => unreachable!(),
    }
}

fn planned_project(input: &str) -> Res<&str, PPlannedProject> {
    let (i, name) = terminated(non_space_or_unix_eol, line_ending)(input)?;
    let (i, roles) = terminated(
        separated_list1(single_space, non_space_or_unix_eol),
        line_ending,
    )(i)?;
    Ok((
        i,
        PPlannedProject {
            name: name.to_string(),
            contributor_names: roles.iter().map(|s| String::from(*s)).collect(),
        },
    ))
}

fn _parse_output(input: &str) -> Res<&str, POutput> {
    let (i, n_projects) = terminated(positive_number, line_ending)(input)?;
    let (i, projects) = many_m_n(n_projects, n_projects, planned_project)(i)?;
    Ok((
        i,
        POutput {
            n_projects,
            projects,
        },
    ))
}

pub fn parse_output(s: &str) -> anyhow::Result<POutput> {
    match _parse_output(s) {
        Ok((_, data)) => Ok(data),
        Err(nom::Err::Error(err)) => bail!("{}", convert_error(s, err.clone())),
        _ => unreachable!(),
    }
}
