#![cfg(feature = "test-sbf")]

use solana_program_test::*;

mod program_test;

use program_test::*;
use spl_governance::{
    error::GovernanceError,
    state::{
        enums::{ProposalState, VoteThreshold},
        proposal::{OptionVoteResult, VoteType},
        vote_record::{Vote, VoteChoice},
    },
};

#[tokio::test]
async fn test_vote_multi_weighted_choice_proposal_non_executable() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let realm_cookie = governance_test.with_realm().await;
    let governed_account_cookie = governance_test.with_governed_account().await;

    let token_owner_record_cookie = governance_test
        .with_community_token_deposit(&realm_cookie)
        .await
        .unwrap();

    let mut governance_config = governance_test.get_default_governance_config();
    governance_config.community_vote_threshold = VoteThreshold::YesVotePercentage(30);

    let mut governance_cookie = governance_test
        .with_governance_using_config(
            &realm_cookie,
            &governed_account_cookie,
            &token_owner_record_cookie,
            &governance_config,
        )
        .await
        .unwrap();

    let proposal_cookie = governance_test
        .with_multi_option_proposal(
            &token_owner_record_cookie,
            &mut governance_cookie,
            vec![
                "option 1".to_string(),
                "option 2".to_string(),
                "option 3".to_string(),
                "option 4".to_string(),
            ],
            false,
            VoteType::MultiWeightedChoice {
                max_winning_options: 4,
                max_voter_options: 4,
            },
        )
        .await
        .unwrap();

    let signatory_record_cookie = governance_test
        .with_signatory(&proposal_cookie, &token_owner_record_cookie)
        .await
        .unwrap();

    let clock = governance_test.bench.get_clock().await;

    governance_test
        .sign_off_proposal(&proposal_cookie, &signatory_record_cookie)
        .await
        .unwrap();

    let vote = Vote::Approve(vec![
        VoteChoice {
            rank: 0,
            weight_percentage: 30,
        },
        VoteChoice {
            rank: 0,
            weight_percentage: 29,
        },
        VoteChoice {
            rank: 0,
            weight_percentage: 41,
        },
        VoteChoice {
            rank: 0,
            weight_percentage: 0,
        },
    ]);

    // Act
    governance_test
        .with_cast_vote(&proposal_cookie, &token_owner_record_cookie, vote)
        .await
        .unwrap();

    // Advance timestamp past max_voting_time
    governance_test
        .advance_clock_past_timestamp(
            governance_cookie.account.config.max_voting_time as i64 + clock.unix_timestamp,
        )
        .await;

    governance_test
        .finalize_vote(&realm_cookie, &proposal_cookie, None)
        .await
        .unwrap();

    // Assert
    let proposal_account = governance_test
        .get_proposal_account(&proposal_cookie.address)
        .await;

    assert_eq!(
        OptionVoteResult::Succeeded,
        proposal_account.options[0].vote_result
    );
    assert_eq!(
        OptionVoteResult::Defeated,
        proposal_account.options[1].vote_result
    );
    assert_eq!(
        OptionVoteResult::Succeeded,
        proposal_account.options[2].vote_result
    );
    assert_eq!(
        OptionVoteResult::Defeated,
        proposal_account.options[3].vote_result
    );
    assert_eq!(
        (token_owner_record_cookie.token_source_amount as f32 * 0.3) as u64,
        proposal_account.options[0].vote_weight
    );
    assert_eq!(
        (token_owner_record_cookie.token_source_amount as f32 * 0.29) as u64,
        proposal_account.options[1].vote_weight
    );
    assert_eq!(
        (token_owner_record_cookie.token_source_amount as f32 * 0.41) as u64,
        proposal_account.options[2].vote_weight
    );
    assert_eq!(0_u64, proposal_account.options[3].vote_weight);

    // None executable proposal transitions to Completed when vote is finalized
    assert_eq!(ProposalState::Completed, proposal_account.state);
}

#[tokio::test]
async fn test_vote_multi_weighted_choice_proposal_with_partial_success() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let realm_cookie = governance_test.with_realm().await;
    let governed_mint_cookie = governance_test.with_governed_mint().await;

    // 100 tokens each, sum 300 tokens
    let token_owner_record_cookie1 = governance_test
        .with_community_token_deposit(&realm_cookie)
        .await
        .unwrap();
    let token_owner_record_cookie2 = governance_test
        .with_community_token_deposit(&realm_cookie)
        .await
        .unwrap();
    let token_owner_record_cookie3 = governance_test
        .with_community_token_deposit(&realm_cookie)
        .await
        .unwrap();

    // 60 tokes approval quorum as 20% of 300 is 60
    let mut governance_config = governance_test.get_default_governance_config();
    governance_config.community_vote_threshold = VoteThreshold::YesVotePercentage(20);

    let mut governance_cookie = governance_test
        .with_mint_governance_using_config(
            &realm_cookie,
            &governed_mint_cookie,
            &token_owner_record_cookie1,
            &governance_config,
        )
        .await
        .unwrap();

    let mut proposal_cookie = governance_test
        .with_multi_option_proposal(
            &token_owner_record_cookie1,
            &mut governance_cookie,
            vec![
                "option 1".to_string(),
                "option 2".to_string(),
                "option 3".to_string(),
                "option 4".to_string(),
            ],
            true,
            VoteType::MultiWeightedChoice {
                max_winning_options: 4,
                max_voter_options: 4,
            },
        )
        .await
        .unwrap();

    let proposal_transaction_cookie1 = governance_test
        .with_mint_tokens_transaction(
            &governed_mint_cookie,
            &mut proposal_cookie,
            &token_owner_record_cookie1,
            0,
            Some(0),
            None,
        )
        .await
        .unwrap();
    let proposal_transaction_cookie2 = governance_test
        .with_mint_tokens_transaction(
            &governed_mint_cookie,
            &mut proposal_cookie,
            &token_owner_record_cookie1,
            1,
            Some(0),
            None,
        )
        .await
        .unwrap();
    let proposal_transaction_cookie3 = governance_test
        .with_mint_tokens_transaction(
            &governed_mint_cookie,
            &mut proposal_cookie,
            &token_owner_record_cookie1,
            2,
            Some(0),
            None,
        )
        .await
        .unwrap();
    let proposal_transaction_cookie4 = governance_test
        .with_mint_tokens_transaction(
            &governed_mint_cookie,
            &mut proposal_cookie,
            &token_owner_record_cookie1,
            3,
            Some(0),
            None,
        )
        .await
        .unwrap();

    let signatory_record_cookie = governance_test
        .with_signatory(&proposal_cookie, &token_owner_record_cookie1)
        .await
        .unwrap();

    governance_test
        .sign_off_proposal(&proposal_cookie, &signatory_record_cookie)
        .await
        .unwrap();

    // vote1:
    //   deny: 100
    // vote2 + vote3:
    //   choice 1: 0 -> Defeated
    //   choice 2: 91 -> Defeated (91 is over 60, 20% from 300, but deny overrules)
    //   choice 3: 101 -> Success
    //   choice 4: 8 -> Defeated (below of 60)

    let vote1 = Vote::Approve(vec![
        VoteChoice {
            rank: 0,
            weight_percentage: 0,
        },
        VoteChoice {
            rank: 0,
            weight_percentage: 30,
        },
        VoteChoice {
            rank: 0,
            weight_percentage: 70,
        },
        VoteChoice {
            rank: 0,
            weight_percentage: 0,
        },
    ]);
    governance_test
        .with_cast_vote(&proposal_cookie, &token_owner_record_cookie1, vote1)
        .await
        .expect("Voting the vote 1 of owner 1 should succeed");

    let vote2 = Vote::Approve(vec![
        VoteChoice {
            rank: 0,
            weight_percentage: 0,
        },
        VoteChoice {
            rank: 0,
            weight_percentage: 61,
        },
        VoteChoice {
            rank: 0,
            weight_percentage: 31,
        },
        VoteChoice {
            rank: 0,
            weight_percentage: 8,
        },
    ]);
    governance_test
        .with_cast_vote(&proposal_cookie, &token_owner_record_cookie2, vote2)
        .await
        .expect("Voting the vote 1 of owner 1 should succeed");

    governance_test
        .with_cast_vote(&proposal_cookie, &token_owner_record_cookie3, Vote::Deny)
        .await
        .expect("Casting deny vote of owner 3 should succeed");

    // Advance timestamp past max_voting_time
    governance_test
        .advance_clock_by_min_timespan(governance_cookie.account.config.max_voting_time as u64)
        .await;
    governance_test
        .finalize_vote(&realm_cookie, &proposal_cookie, None)
        .await
        .unwrap();
    // Advance timestamp past hold_up_time
    governance_test
        .advance_clock_by_min_timespan(proposal_transaction_cookie1.account.hold_up_time as u64)
        .await;

    let mut proposal_account = governance_test
        .get_proposal_account(&proposal_cookie.address)
        .await;

    assert_eq!(ProposalState::Succeeded, proposal_account.state);

    // Act
    let transaction1_err = governance_test
        .execute_proposal_transaction(&proposal_cookie, &proposal_transaction_cookie1)
        .await
        .expect_err("Choice 1 should fail to execute, it hasn't got enough votes");
    let transaction2_err = governance_test
        .execute_proposal_transaction(&proposal_cookie, &proposal_transaction_cookie2)
        .await
        .expect_err("Choice 2 should fail to execute, it hasn't got enough votes");
    governance_test
        .execute_proposal_transaction(&proposal_cookie, &proposal_transaction_cookie3)
        .await
        .expect("Choice 3 should be executed as it won the poll");
    let transaction4_err = governance_test
        .execute_proposal_transaction(&proposal_cookie, &proposal_transaction_cookie4)
        .await
        .expect_err("Choice 4 should be executed as the winner has been executed already");

    // Assert
    proposal_account = governance_test
        .get_proposal_account(&proposal_cookie.address)
        .await;

    assert_eq!(ProposalState::Completed, proposal_account.state);

    assert_eq!(
        transaction1_err,
        GovernanceError::CannotExecuteDefeatedOption.into()
    );
    assert_eq!(
        transaction2_err,
        GovernanceError::CannotExecuteDefeatedOption.into()
    );
    assert_eq!(
        transaction4_err,
        GovernanceError::InvalidStateCannotExecuteTransaction.into()
    );
}
