use std::process::{Command, exit};
use names::Generator;

fn get_current_branch() -> String {
    let branch_command = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .expect("Failed to get current branch");

    String::from_utf8_lossy(&branch_command.stdout)
        .trim()
        .to_string()
}

fn check_git_repo() -> bool {
    Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn update_commit_push() {
    if !check_git_repo() {
        eprintln!("Error: Not in a git repository");
        exit(1);
    }

    let add_command = Command::new("git")
        .args(["add", "."])
        .output()
        .expect("Failed to execute git add");

    if !add_command.status.success() {
        eprintln!("Failed to execute git add: {}", 
            String::from_utf8_lossy(&add_command.stderr));
        exit(1);
    }

    let commit_message = name_generator();
    let commit_command = Command::new("git")
        .args(["commit", "-m", &commit_message])
        .output()
        .expect("Failed to execute git commit");

    if !commit_command.status.success() {
        eprintln!("Failed to execute git commit: {}", 
            String::from_utf8_lossy(&commit_command.stderr));
        exit(1);
    }

    let current_branch = get_current_branch();
    let push_command = Command::new("git")
        .args(["push", "origin", &current_branch])
        .output()
        .expect("Failed to execute git push");

    if !push_command.status.success() {
        eprintln!("Failed to execute git push: {}", 
            String::from_utf8_lossy(&push_command.stderr));
        exit(1);
    }

    println!("Successfully added, committed, and pushed changes");
}

fn name_generator() -> String {
    let mut generator = Generator::default();
    generator.next().unwrap()
}

fn main(){
    update_commit_push();
}

