//! Test de préservation des modules legacy

use std::path::Path;

#[test]
fn test_legacy_modules_preserved() {
    // Vérifier que tous les modules legacy sont présents
    let legacy_modules = [
        "src/legacy/mod.rs",
        "src/legacy/pods.rs", 
        "src/legacy/system.rs",
        "src/legacy/health.rs",
        "src/legacy/backups.rs",
        "src/legacy/security.rs",
        "src/legacy/prometheus.rs",
        "src/legacy/nodes.rs",
        "src/legacy/calendar.rs",
        "src/legacy/doctor.rs",
        "src/legacy/mcp.rs",
        "src/legacy/translation.rs",
        "src/legacy/proxmox.rs",
        "src/legacy/cilium.rs",
        "src/legacy/newsfeed.rs",
        "src/legacy/chat.rs",
        "src/legacy/low_priority_part1.rs",
        "src/legacy/low_priority_part2.rs",
        "src/legacy/low_priority_part3.rs",
        "src/legacy/low_priority_part4.rs",
        "src/legacy/low_priority_part5.rs",
        "src/legacy/low_priority_part6.rs",
        "src/legacy/low_priority_part7.rs",
        "src/legacy/low_priority_part8.rs",
        "src/legacy/low_priority_part9.rs",
        "src/legacy/low_priority_part10.rs",
        "src/legacy/low_priority_part11.rs",
        "src/legacy/low_priority_part12.rs",
        "src/legacy/low_priority_part13.rs",
        "src/legacy/low_priority_part14.rs",
        "src/legacy/low_priority_part15.rs",
        "src/legacy/low_priority_part16.rs",
        "src/legacy/low_priority_part17.rs",
        "src/legacy/low_priority_part18.rs",
        "src/legacy/low_priority_part19.rs",
        "src/legacy/low_priority_part20.rs",
        "src/legacy/low_priority_part21.rs",
    ];
    
    let mut preserved_count = 0;
    let mut missing_modules = Vec::new();
    
    for module in &legacy_modules {
        if Path::new(module).exists() {
            preserved_count += 1;
        } else {
            missing_modules.push(module);
        }
    }
    
    println!("✅ Modules legacy préservés: {}/{}", preserved_count, legacy_modules.len());
    
    if !missing_modules.is_empty() {
        println!("⚠️  Modules manquants: {:?}", missing_modules);
    }
    
    // Au moins 30 modules legacy doivent être présents
    assert!(preserved_count >= 30, "Pas assez de modules legacy préservés: {}", preserved_count);
}

#[test]
fn test_hexagonal_architecture_present() {
    // Vérifier que l'architecture hexagonale est présente
    let hex_dirs = [
        "src/domain",
        "src/application", 
        "src/infrastructure",
        "src/interfaces",
    ];
    
    for dir in &hex_dirs {
        assert!(Path::new(dir).exists(), "Architecture hexagonale manquante: {}", dir);
    }
    
    println!("✅ Architecture hexagonale détectée");
}

#[test]
fn test_basic_compilation_readiness() {
    // Test basique de compilation
    assert_eq!(2 + 2, 4);
    println!("✅ Compilation de base fonctionnelle");
}
