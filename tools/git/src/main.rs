use std::env;
use git2::{Repository, IndexAddOption, Signature};

fn main() -> anyhow::Result<()> {
    let args = env::args().collect::<Vec<String>>();

    if args.len() <= 2 {
        eprintln!("Usage: {} <branch name> <commit message>", args[0]);
        std::process::exit(1);
    }

    let branch_name = &args[1];
    let commit_message = &args[2];

    let repo = Repository::open(".")?;

    // Automatically fetch the user's name and email from Git configuration
    let config = repo.config()?;
    let name = config.get_string("user.name")?;
    let email = config.get_string("user.email")?;

    let head_ref = repo.head()?.target().unwrap();
    let commit = repo.find_commit(head_ref)?;

    repo.branch(branch_name, &commit, false)?;
    let refname = format!("refs/heads/{}", branch_name);
    let obj = repo.revparse_single(&refname)?;
    repo.checkout_tree(&obj, None)?;
    repo.set_head(&refname)?;

    let mut index = repo.index()?;
    index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)?;
    index.write()?;

    // Create the signature with the fetched name and email
    let signature = Signature::now(&name, &email)?;
    let oid = index.write_tree()?;
    let parent_commit = repo.find_commit(repo.head()?.target().unwrap())?;
    let tree = repo.find_tree(oid)?;

    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        commit_message,
        &tree,
        &[&parent_commit],
    )?;

    Ok(())
}
