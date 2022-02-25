#[derive(Debug, PartialEq)]
pub struct PContributorSkill {
    pub name: String,
    pub level: usize,
}

#[derive(Debug, PartialEq)]
pub struct PContributor {
    pub name: String,
    pub n_skills: usize,
    pub skills: Vec<PContributorSkill>,
}

#[derive(Debug, PartialEq)]
pub struct PProject {
    pub name: String,
    pub days_to_completion: usize,
    pub score: usize,
    pub best_before: usize,
    pub n_roles: usize,
    pub skills: Vec<PContributorSkill>
}

#[derive(Debug, PartialEq)]
pub struct PInput {
    pub n_contributors: usize,
    pub n_projects: usize,
    pub contributors: Vec<PContributor>,
    pub projects: Vec<PProject>,
}