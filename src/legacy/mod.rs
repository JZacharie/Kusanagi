//! Legacy Modules
//!
//! These modules are in the process of being refactored to hexagonal architecture.
//! They are organized here to keep the src/ root clean.

pub mod alertmanager;
pub mod apps;
pub mod argocd;
pub mod backups;
pub mod calendar;
pub mod chat;
pub mod chat_storage;
pub mod cilium;
pub mod cluster;
pub mod database;
pub mod doctor;
pub mod events;
pub mod export;
pub mod health;
pub mod homeassistant;
pub mod ingress;
pub mod llm;
pub mod mcp;
pub mod mqtt;
pub mod newsfeed;
pub mod nodes;
pub mod notifications;
pub mod pods;
pub mod prometheus;
pub mod proxmox;
pub mod quota;
pub mod security;
pub mod services;
pub mod setup;
pub mod slack;
pub mod storage;
pub mod system;
pub mod telemetry;
pub mod translation;
pub mod weather;
pub mod ws;
