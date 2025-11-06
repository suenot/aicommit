async fn watch_and_commit(config: &Config, cli: &Cli) -> Result<(), String> {
    let wait_for_edit = cli.wait_for_edit.as_ref()
        .map(|w| parse_duration(w))
        .transpose()?;

    info!("Watching for changes...");
    if let Some(delay) = wait_for_edit {
        info!("Waiting {:?} after edits before committing", delay);
    }

    // Initialize waiting list for files with their last modification timestamps
    let mut waiting_files: std::collections::HashMap<String, std::time::Instant> = std::collections::HashMap::new();
    
    // Отслеживание хешей содержимого файлов для определения реальных изменений
    let mut file_hashes: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    
    loop {
        // Sleep for a short period to reduce CPU usage
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        
        // Get list of modified files
        let output = Command::new("sh")
            .arg("-c")
            .arg("git ls-files -m -o --exclude-standard")
            .output()
            .map_err(|e| format!("Failed to check modified files: {}", e))?;

        if !output.status.success() {
            continue;
        }

        let modified_files = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        // Check if we have any new modified files
        if !modified_files.is_empty() {
            for file in &modified_files {
                // Проверяем, действительно ли содержимое файла изменилось
                // Получаем хеш содержимого файла
                let hash_output = Command::new("sh")
                    .arg("-c")
                    .arg(&format!("git hash-object \"{}\"", file.replace("\"", "\\\"")))
                    .output();
                
                match hash_output {
                    Ok(output) if output.status.success() => {
                        let new_hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
                        let old_hash = file_hashes.get(file).cloned().unwrap_or_default();
                        
                        // Проверяем, изменился ли хеш файла
                        let is_real_change = new_hash != old_hash;
                        
                        // Обновляем сохраненный хеш файла
                        file_hashes.insert(file.clone(), new_hash);
                        
                        // Если файл действительно изменился, обрабатываем изменение
                        if is_real_change {
                            // Log the change
                            println!("File changed: {}", file);
                            
                            if let Some(delay) = wait_for_edit {
                                // Check if file is already in waiting list
                                if waiting_files.contains_key(file) {
                                    // Reset timer for this file
                                    let _ready_time = std::time::Instant::now() + delay;
                                    let ready_time_str = chrono::Local::now().checked_add_signed(chrono::Duration::from_std(delay).unwrap_or_default())
                                        .map(|dt| dt.format("%H:%M:%S").to_string())
                                        .unwrap_or_else(|| "unknown time".to_string());
                                    
                                    println!("Resetting timer for file: {} (will be ready at {})", file, ready_time_str);
                                    waiting_files.insert(file.clone(), std::time::Instant::now());
                                } else {
                                    // Add file to waiting list with current timestamp
                                    let _ready_time = std::time::Instant::now() + delay;
                                    let ready_time_str = chrono::Local::now().checked_add_signed(chrono::Duration::from_std(delay).unwrap_or_default())
                                        .map(|dt| dt.format("%H:%M:%S").to_string())
                                        .unwrap_or_else(|| "unknown time".to_string());
                                    
                                    println!("Adding file to waiting list: {} (will be ready at {})", file, ready_time_str);
                                    waiting_files.insert(file.clone(), std::time::Instant::now());
                                }
                            } else {
                                // If no wait-for-edit delay specified, immediately add the file
                                let git_add = Command::new("sh")
                                    .arg("-c")
                                    .arg(&format!("git add \"{}\"", file.replace("\"", "\\\"")))
                                    .output()
                                    .map_err(|e| format!("Failed to add file: {}", e))?;

                                if !git_add.status.success() {
                                    println!("Failed to add file: {}", String::from_utf8_lossy(&git_add.stderr));
                                }
                                
                                // If there are changes to commit, do it immediately
                                match get_git_diff(cli) {
                                    Ok(diff) if !diff.is_empty() => {
                                        match run_commit(config, cli).await {
                                            Ok(_) => {
                                                println!("\nCommitted changes.");
                                                println!("Continuing to watch for changes...");
                                            }
                                            Err(e) => println!("Failed to commit: {}", e),
                                        }
                                    }
                                    _ => {} // No changes or error, continue watching
                                }
                            }
                        }
                    },
                    _ => {
                        // Если не удалось получить хеш, обрабатываем как изменение
                        // для новых файлов это нормально
                        println!("File changed: {}", file);
                        
                        if let Some(delay) = wait_for_edit {
                            if waiting_files.contains_key(file) {
                                let _ready_time_str = chrono::Local::now().checked_add_signed(chrono::Duration::from_std(delay).unwrap_or_default())
                                    .map(|dt| dt.format("%H:%M:%S").to_string())
                                    .unwrap_or_else(|| "unknown time".to_string());
                                
                                println!("Resetting timer for file: {} (will be ready at {})", file, _ready_time_str);
                                waiting_files.insert(file.clone(), std::time::Instant::now());
                            } else {
                                let _ready_time_str = chrono::Local::now().checked_add_signed(chrono::Duration::from_std(delay).unwrap_or_default())
                                    .map(|dt| dt.format("%H:%M:%S").to_string())
                                    .unwrap_or_else(|| "unknown time".to_string());
                                
                                println!("Adding file to waiting list: {} (will be ready at {})", file, _ready_time_str);
                                waiting_files.insert(file.clone(), std::time::Instant::now());
                            }
                        } else {
                            let git_add = Command::new("sh")
                                .arg("-c")
                                .arg(&format!("git add \"{}\"", file.replace("\"", "\\\"")))
                                .output()
                                .map_err(|e| format!("Failed to add file: {}", e))?;

                            if !git_add.status.success() {
                                println!("Failed to add file: {}", String::from_utf8_lossy(&git_add.stderr));
                            }
                            
                            match get_git_diff(cli) {
                                Ok(diff) if !diff.is_empty() => {
                                    match run_commit(config, cli).await {
                                        Ok(_) => {
                                            println!("\nCommitted changes.");
                                            println!("Continuing to watch for changes...");
                                        }
                                        Err(e) => println!("Failed to commit: {}", e),
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        // If wait-for-edit is specified, check the waiting list
        if let Some(delay) = wait_for_edit {
            let now = std::time::Instant::now();
            let mut files_to_commit = Vec::new();
            
            // Check each file in the waiting list
            for (file, timestamp) in &waiting_files {
                if now.duration_since(*timestamp) >= delay {
                    // File has been stable for the specified time, add it to git
                    files_to_commit.push(file.clone());
                }
            }
            
            // Add stable files to git and commit
            if !files_to_commit.is_empty() {
                for file in &files_to_commit {
                    println!("File ready for commit: {} (stable for {:?})", file, delay);
                    
                    let git_add = Command::new("sh")
                        .arg("-c")
                        .arg(&format!("git add \"{}\"", file.replace("\"", "\\\"")))
                        .output()
                        .map_err(|e| format!("Failed to add file: {}", e))?;

                    if !git_add.status.success() {
                        println!("Failed to add file: {}", String::from_utf8_lossy(&git_add.stderr));
                    }
                }
                
                // Commit the changes
                match get_git_diff(cli) {
                    Ok(diff) if !diff.is_empty() => {
                        match run_commit(config, cli).await {
                            Ok(_) => {
                                println!("\nCommitted changes for stable files.");
                                println!("Continuing to watch for changes...");
                                
                                // Remove committed files from waiting list
                                for file in files_to_commit {
                                    waiting_files.remove(&file);
                                    // Также обновляем хеш после коммита
                                    if let Ok(output) = Command::new("sh")
                                        .arg("-c")
                                        .arg(&format!("git hash-object \"{}\"", file.replace("\"", "\\\"")))
                                        .output() {
                                        if output.status.success() {
                                            let new_hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
                                            file_hashes.insert(file, new_hash);
                                        }
                                    }
                                }
                            }
                            Err(e) => println!("Failed to commit: {}", e),
                        }
                    }
                    _ => {} // No changes or error, continue watching
                }
            }
        }
    }
}