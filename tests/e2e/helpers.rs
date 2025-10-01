use std::process::Command;
use tempfile::TempDir;

pub struct TestProject {
    pub dir: TempDir,
    pub binary_path: String,
}

impl TestProject {
    pub fn new() -> Self {
        let dir = TempDir::new().expect("Failed to create temp dir");
        let binary_path = env!("CARGO_BIN_EXE_py-license-auditor").to_string();
        
        Self { dir, binary_path }
    }
    
    pub fn init_uv_project(&self, name: &str, deps: &[&str]) -> std::io::Result<()> {
        let project_path = self.dir.path().join(name);
        
        // Create uv project
        Command::new("uv")
            .args(["init", name])
            .current_dir(self.dir.path())
            .output()?;
            
        // Add dependencies
        if !deps.is_empty() {
            let mut cmd = Command::new("uv");
            cmd.arg("add").args(deps).current_dir(&project_path);
            cmd.output()?;
        }
        
        Ok(())
    }
    
    pub fn run_auditor(&self, project: &str, args: &[&str]) -> std::process::Output {
        let project_path = self.dir.path().join(project);
        
        Command::new(&self.binary_path)
            .args(args)
            .current_dir(project_path)
            .output()
            .expect("Failed to run py-license-auditor")
    }
}
