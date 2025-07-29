use chrono::{DateTime, Local};
use git2::{Repository, Signature, Time, Oid, RemoteCallbacks, PushOptions, Cred};
use std::env;
use crate::patterns::CommitInfo;
use crate::error::Result;

pub struct GitOperations {
    repo: Repository,
}

impl GitOperations {
    pub fn new(repo: Repository) -> Self {
        Self { repo }
    }
    
    pub fn get_latest_autogen_commit(&mut self) -> Result<Option<DateTime<Local>>> {
        let mut revwalk = self.repo.revwalk()?;
        revwalk.push_head()?;
        
        for oid in revwalk {
            let oid = oid?;
            let commit = self.repo.find_commit(oid)?;
            
            if let Some(message) = commit.message() {
                if message.starts_with("[AutoGen]") {
                    let time = commit.time();
                    let timestamp = time.seconds();
                    let datetime = DateTime::from_timestamp(timestamp, 0)
                        .unwrap()
                        .with_timezone(&Local);
                    return Ok(Some(datetime));
                }
            }
        }
        
        Ok(None)
    }
    
    pub fn create_commit(&mut self, commit_info: &CommitInfo) -> Result<Oid> {
        // Ensure we're on main branch
        self.ensure_main_branch()?;
        
        // Get current tree (we'll create empty commits like --allow-empty)
        let tree = match self.repo.head() {
            Ok(head) => {
                let commit = head.peel_to_commit()?;
                commit.tree()?
            }
            Err(_) => {
                // If no HEAD, create empty tree
                let tree_id = self.repo.treebuilder(None)?.write()?;
                self.repo.find_tree(tree_id)?
            }
        };
        
        // Get parent commit
        let parent_commit = match self.repo.head() {
            Ok(head) => {
                let oid = head.target().unwrap();
                Some(self.repo.find_commit(oid)?)
            }
            Err(_) => None,
        };
        
        // Create signature with commit date
        let sig = Signature::new(
            "GitHub Grid", 
            "github-grid@example.com",
            &Time::new(commit_info.date.timestamp(), 0)
        )?;
        
        // Create empty commit (like git commit --allow-empty)
        let parents: Vec<_> = parent_commit.iter().collect();
        let commit_id = self.repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            &commit_info.message,
            &tree,
            &parents,
        )?;
        
        Ok(commit_id)
    }
    
    pub fn push_commits(&mut self) -> Result<()> {
        let mut remote = self.repo.find_remote("origin")?;
        let mut callbacks = RemoteCallbacks::new();
        
        // Try to get credentials in order of preference
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            // Try GitHub token first
            if let Ok(token) = env::var("GITHUB_TOKEN") {
                return Cred::userpass_plaintext(
                    username_from_url.unwrap_or("git"),
                    &token
                );
            }
            
            // Try gh CLI token
            if let Ok(output) = std::process::Command::new("gh")
                .args(&["auth", "token"])
                .output() 
            {
                if output.status.success() {
                    let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    return Cred::userpass_plaintext(
                        username_from_url.unwrap_or("git"),
                        &token
                    );
                }
            }
            
            // Fallback to SSH key
            Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
        });
        
        let mut opts = PushOptions::new();
        opts.remote_callbacks(callbacks);
        
        remote.push(&["refs/heads/main:refs/heads/main"], Some(&mut opts))?;
        Ok(())
    }
    
    fn ensure_main_branch(&mut self) -> Result<()> {
        let head = self.repo.head()?;
        let branch_name = head.shorthand().unwrap_or("");
        
        if branch_name != "main" {
            // Try to checkout main branch
            let obj = self.repo.revparse_single("refs/heads/main")?;
            self.repo.checkout_tree(&obj, None)?;
            self.repo.set_head("refs/heads/main")?;
        }
        
        Ok(())
    }
    
}