use crate::prelude::*;

/// The states that are truly ir-reconcilable, that fully clear all affected entities and are a major `.run_if()` condition
#[derive(States, Debug, Default, Hash, PartialEq, Eq, Clone, Copy)]
pub enum GameState {
	#[default]
	StartMenu,
	
	InGame,
}