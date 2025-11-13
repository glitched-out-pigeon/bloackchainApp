#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Symbol, String, symbol_short, Address};

// Structure to track overall voting statistics
#[contracttype]
#[derive(Clone)]
pub struct VotingStats {
    pub total_proposals: u64,
    pub active_proposals: u64,
    pub closed_proposals: u64,
}

// Reference symbol for voting statistics
const VOTING_STATS: Symbol = symbol_short!("V_STATS");

// Counter for generating unique proposal IDs
const PROPOSAL_COUNT: Symbol = symbol_short!("P_COUNT");

// Mapping for proposals
#[contracttype]
pub enum ProposalBook {
    Proposal(u64)
}

// Structure representing a voting proposal
#[contracttype]
#[derive(Clone)]
pub struct Proposal {
    pub proposal_id: u64,
    pub title: String,
    pub description: String,
    pub yes_votes: u64,
    pub no_votes: u64,
    pub created_time: u64,
    pub is_active: bool,
}

// Mapping to track if an address has voted on a proposal
#[contracttype]
pub enum VoteRecord {
    HasVoted(u64, Address) // (proposal_id, voter_address)
}

#[contract]
pub struct VotingContract;

#[contractimpl]
impl VotingContract {
    
    // Function 1: Create a new voting proposal
    pub fn create_proposal(env: Env, title: String, description: String) -> u64 {
        let mut proposal_count: u64 = env.storage().instance().get(&PROPOSAL_COUNT).unwrap_or(0);
        proposal_count += 1;
        
        let time = env.ledger().timestamp();
        let mut stats = Self::get_voting_stats(env.clone());
        
        // Create new proposal
        let new_proposal = Proposal {
            proposal_id: proposal_count,
            title,
            description,
            yes_votes: 0,
            no_votes: 0,
            created_time: time,
            is_active: true,
        };
        
        // Update statistics
        stats.total_proposals += 1;
        stats.active_proposals += 1;
        
        // Store proposal and updated stats
        env.storage().instance().set(&ProposalBook::Proposal(proposal_count), &new_proposal);
        env.storage().instance().set(&VOTING_STATS, &stats);
        env.storage().instance().set(&PROPOSAL_COUNT, &proposal_count);
        
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Proposal Created with ID: {}", proposal_count);
        proposal_count
    }
    
    // Function 2: Cast a vote on a proposal
    pub fn cast_vote(env: Env, proposal_id: u64, vote_yes: bool, voter: Address) {
        voter.require_auth();
        
        let vote_key = VoteRecord::HasVoted(proposal_id, voter.clone());
        
        // Check if voter has already voted
        let has_voted: bool = env.storage().instance().get(&vote_key).unwrap_or(false);
        
        if has_voted {
            log!(&env, "Address has already voted on this proposal");
            panic!("Already voted!");
        }
        
        // Get proposal
        let mut proposal = Self::get_proposal(env.clone(), proposal_id);
        
        if !proposal.is_active {
            log!(&env, "Proposal is not active");
            panic!("Proposal closed!");
        }
        
        // Record vote
        if vote_yes {
            proposal.yes_votes += 1;
            log!(&env, "YES vote recorded for Proposal ID: {}", proposal_id);
        } else {
            proposal.no_votes += 1;
            log!(&env, "NO vote recorded for Proposal ID: {}", proposal_id);
        }
        
        // Mark voter as having voted
        env.storage().instance().set(&vote_key, &true);
        
        // Update proposal
        env.storage().instance().set(&ProposalBook::Proposal(proposal_id), &proposal);
        
        env.storage().instance().extend_ttl(5000, 5000);
    }
    
    // Function 3: Close a proposal (admin function)
    pub fn close_proposal(env: Env, proposal_id: u64) {
        let mut proposal = Self::get_proposal(env.clone(), proposal_id);
        
        if !proposal.is_active {
            log!(&env, "Proposal is already closed");
            panic!("Already closed!");
        }
        
        proposal.is_active = false;
        
        let mut stats = Self::get_voting_stats(env.clone());
        stats.active_proposals -= 1;
        stats.closed_proposals += 1;
        
        // Update proposal and stats
        env.storage().instance().set(&ProposalBook::Proposal(proposal_id), &proposal);
        env.storage().instance().set(&VOTING_STATS, &stats);
        
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Proposal ID: {} has been closed", proposal_id);
    }
    
    // Function 4: View proposal details
    pub fn get_proposal(env: Env, proposal_id: u64) -> Proposal {
        let key = ProposalBook::Proposal(proposal_id);
        
        env.storage().instance().get(&key).unwrap_or(Proposal {
            proposal_id: 0,
            title: String::from_str(&env, "Not_Found"),
            description: String::from_str(&env, "Not_Found"),
            yes_votes: 0,
            no_votes: 0,
            created_time: 0,
            is_active: false,
        })
    }
    
    // Helper function: Get voting statistics
    pub fn get_voting_stats(env: Env) -> VotingStats {
        env.storage().instance().get(&VOTING_STATS).unwrap_or(VotingStats {
            total_proposals: 0,
            active_proposals: 0,
            closed_proposals: 0,
        })
    }
}