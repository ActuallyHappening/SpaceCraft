//! Global state management

use crate::prelude::*;

/// The states that are truly ir-reconcilable, that fully clear all affected entities and are a major `.run_if()` condition
#[derive(States, Default, Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum GlobalGameStates {
	#[default]
	StartMenu,

	InGame,
}
