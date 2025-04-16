use std::fs;
use std::path::{Path, PathBuf};
use std::io;
use serde::{Serialize, Deserialize};
use serde_json;

use crate::models::project::Project;

/// Error types for project storage operations
#[derive(Debug)]
pub enum StorageError {
    IoError(io::Error),
    SerializationError(serde_json::Error),
    ProjectNotFound,
    InvalidProjectData,
}

impl From<io::Error> for StorageError {
    fn from(error: io::Error) -> Self {
        StorageError::IoError(error)
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(error: serde_json::Error) -> Self {
        StorageError::SerializationError(error)
    }
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::IoError(e) => write!(f, "I/O error: {}", e),
            StorageError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            StorageError::ProjectNotFound => write!(f, "Project not found"),
            StorageError::InvalidProjectData => write!(f, "Invalid project data"),
        }
    }
}

impl std::error::Error for StorageError {}

/// Metadata about a stored project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMeta {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub version: String,
    pub created_at: String,
    pub updated_at: String,
    pub file_path: PathBuf,
}

impl From<&Project> for ProjectMeta {
    fn from(project: &Project) -> Self {
        ProjectMeta {
            id: project.id.clone(),
            name: project.name.clone(),
            description: project.description.clone(),
            author: project.author.clone(),
            version: project.version.clone(),
            created_at: project.created_at.clone(),
            updated_at: project.updated_at.clone(),
            file_path: PathBuf::new(), // Will be set by the storage manager
        }
    }
}

/// Project storage manager
pub struct ProjectStorage {
    projects_dir: PathBuf,
}

impl ProjectStorage {
    /// Create a new project storage manager
    pub fn new(projects_dir: PathBuf) -> Result<Self, StorageError> {
        // Create the projects directory if it doesn't exist
        if !projects_dir.exists() {
            fs::create_dir_all(&projects_dir)?;
        }

        Ok(ProjectStorage { projects_dir })
    }

    /// Initialize the storage with default projects directory
    pub fn init() -> Result<Self, StorageError> {
        let mut projects_dir = dirs::data_local_dir()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Could not find local data directory"))?;

        projects_dir.push("snap-blaster");
        projects_dir.push("projects");

        Self::new(projects_dir)
    }

    /// Get the path to a project file
    fn get_project_path(&self, id: &str) -> PathBuf {
        let mut path = self.projects_dir.clone();
        path.push(format!("{}.json", id));
        path
    }

    /// Save a project to storage
    pub fn save_project(&self, project: &Project) -> Result<(), StorageError> {
        // Update timestamp
        let mut project = project.clone();
        project.updated_at = chrono::Utc::now().to_rfc3339();

        // Convert to JSON
        let json = serde_json::to_string_pretty(&project)?;

        // Write to file
        let path = self.get_project_path(&project.id);
        fs::write(path, json)?;

        Ok(())
    }

    /// Load a project from storage
    pub fn load_project(&self, id: &str) -> Result<Project, StorageError> {
        let path = self.get_project_path(id);

        if !path.exists() {
            return Err(StorageError::ProjectNotFound);
        }

        let json = fs::read_to_string(path)?;
        let project = serde_json::from_str(&json)?;

        Ok(project)
    }

    /// Delete a project from storage
    pub fn delete_project(&self, id: &str) -> Result<(), StorageError> {
        let path = self.get_project_path(id);

        if !path.exists() {
            return Err(StorageError::ProjectNotFound);
        }

        fs::remove_file(path)?;
        Ok(())
    }

    /// List all projects in storage
    pub fn list_projects(&self) -> Result<Vec<ProjectMeta>, StorageError> {
        // Create the directory if it doesn't exist
        if !self.projects_dir.exists() {
            fs::create_dir_all(&self.projects_dir)?;
            return Ok(Vec::new());
        }

        let mut projects = Vec::new();

        for entry in fs::read_dir(&self.projects_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Only process JSON files
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }

            if let Ok(json) = fs::read_to_string(&path) {
                if let Ok(project) = serde_json::from_str::<Project>(&json) {
                    let mut meta = ProjectMeta::from(&project);
                    meta.file_path = path;
                    projects.push(meta);
                }
            }
        }

        // Sort by last updated time (newest first)
        projects.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        Ok(projects)
    }

    /// Import a project from a file
    pub fn import_project<P: AsRef<Path>>(&self, path: P) -> Result<Project, StorageError> {
        let json = fs::read_to_string(path)?;
        let mut project: Project = serde_json::from_str(&json)?;

        // Generate a new ID to avoid conflicts
        use uuid::Uuid;
        project.id = Uuid::new_v4().to_string();

        // Save the imported project
        self.save_project(&project)?;

        Ok(project)
    }

    /// Export a project to a file
    pub fn export_project<P: AsRef<Path>>(&self, id: &str, path: P) -> Result<(), StorageError> {
        let project = self.load_project(id)?;
        let json = serde_json::to_string_pretty(&project)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Create a template project
    pub fn create_template_project(&self, name: &str, author: Option<&str>) -> Project {
        let mut project = Project::new(name, author);

        // Add some common CC definitions
        use crate::models::project::CCDefinition;

        // CC7 = Volume
        let mut volume = CCDefinition::new(0, 7, "Volume");
        volume.description = Some("Channel volume".to_string());
        volume.default_value = 100;
        volume.use_transitions = true;
        project.add_cc_definition(volume);

        // CC10 = Pan
        let mut pan = CCDefinition::new(0, 10, "Pan");
        pan.description = Some("Stereo panning position".to_string());
        pan.default_value = 64;
        pan.use_transitions = true;
        project.add_cc_definition(pan);

        // CC74 = Filter Cutoff
        let mut cutoff = CCDefinition::new(0, 74, "Filter Cutoff");
        cutoff.description = Some("Filter cutoff frequency".to_string());
        cutoff.default_value = 127;
        cutoff.use_transitions = true;
        project.add_cc_definition(cutoff);

        // CC71 = Resonance
        let mut resonance = CCDefinition::new(0, 71, "Resonance");
        resonance.description = Some("Filter resonance".to_string());
        resonance.default_value = 0;
        resonance.use_transitions = true;
        project.add_cc_definition(resonance);

        // Add a default scene
        use crate::models::scene::Scene;
        use crate::models::cc::CCValue;

        let mut default_scene = Scene::new("default", "Default");
        default_scene.description = Some("Initial state".to_string());

        // Add CC values to the scene
        let volume_cc = CCValue::new(0, 7, 100);
        let pan_cc = CCValue::new(0, 10, 64);
        let cutoff_cc = CCValue::new(0, 74, 127);
        let resonance_cc = CCValue::new(0, 71, 0);

        default_scene.add_cc(volume_cc);
        default_scene.add_cc(pan_cc);
        default_scene.add_cc(cutoff_cc);
        default_scene.add_cc(resonance_cc);

        project.add_scene(default_scene);

        project
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_save_and_load_project() {
        let temp_dir = tempdir().unwrap();
        let storage = ProjectStorage::new(temp_dir.path().to_path_buf()).unwrap();

        let project = Project::new("Test Project", Some("Test Author"));
        let id = project.id.clone();

        // Save the project
        storage.save_project(&project).unwrap();

        // Load the project
        let loaded = storage.load_project(&id).unwrap();

        assert_eq!(loaded.id, id);
        assert_eq!(loaded.name, "Test Project");
        assert_eq!(loaded.author, Some("Test Author".to_string()));
    }

    #[test]
    fn test_delete_project() {
        let temp_dir = tempdir().unwrap();
        let storage = ProjectStorage::new(temp_dir.path().to_path_buf()).unwrap();

        let project = Project::new("Test Project", Some("Test Author"));
        let id = project.id.clone();

        // Save the project
        storage.save_project(&project).unwrap();

        // Delete the project
        storage.delete_project(&id).unwrap();

        // Try to load the deleted project
        let result = storage.load_project(&id);
        assert!(matches!(result, Err(StorageError::ProjectNotFound)));
    }

    #[test]
    fn test_list_projects() {
        let temp_dir = tempdir().unwrap();
        let storage = ProjectStorage::new(temp_dir.path().to_path_buf()).unwrap();

        // Create a few projects
        let project1 = Project::new("Project 1", None);
        let project2 = Project::new("Project 2", None);
        let project3 = Project::new("Project 3", None);

        // Save the projects
        storage.save_project(&project1).unwrap();
        storage.save_project(&project2).unwrap();
        storage.save_project(&project3).unwrap();

        // List the projects
        let projects = storage.list_projects().unwrap();

        assert_eq!(projects.len(), 3);
    }
}