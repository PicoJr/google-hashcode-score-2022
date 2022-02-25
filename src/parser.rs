use anyhow::bail;
use nom::bytes::complete::{tag, take_while1, take_while_m_n};
use nom::combinator::{map_res, verify};
use nom::error::{context, convert_error, VerboseError};
use nom::multi::{many1, many_m_n, separated_list1};
use nom::sequence::{terminated, tuple};
use nom::IResult;

use crate::data::{PContributor, PContributorSkill, PInput, POutput, PProject, PlannedProject};
use nom::character::complete::line_ending;

pub(crate) type N = usize;
pub(crate) type Res<T, U> = IResult<T, U, VerboseError<T>>;

fn number(input: &str) -> Res<&str, &str> {
    context("number", take_while1(|c: char| c.is_digit(10)))(input)
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

fn str_list_exact(s: &str, expected_size: usize) -> Res<&str, Vec<&str>> {
    verify(
        separated_list1(single_space, non_space_or_unix_eol),
        |s: &[&str]| s.len() == expected_size,
    )(s)
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
        },
    ))
}

fn _parse_input(input: &str) -> Res<&str, PInput> {
    let (i, (n_contributors, _, n_projects)) = terminated(
        tuple((positive_number, single_space, positive_number)),
        line_ending,
    )(input)?;
    let (i, contributors) = many_m_n(n_contributors, n_contributors, contributor)(i)?;
    let (i, projects) = many_m_n(n_projects, n_projects, project)(i)?;
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

pub fn planned_project(input: &str) -> Res<&str, PlannedProject> {
    let (i, name) = terminated(non_space_or_unix_eol, line_ending)(input)?;
    let (i, roles) = terminated(
        separated_list1(single_space, non_space_or_unix_eol),
        line_ending,
    )(i)?;
    Ok((
        i,
        PlannedProject {
            name: name.to_string(),
            roles: roles.iter().map(|s| String::from(*s)).collect(),
        },
    ))
}

pub fn _parse_output(input: &str) -> Res<&str, POutput> {
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
