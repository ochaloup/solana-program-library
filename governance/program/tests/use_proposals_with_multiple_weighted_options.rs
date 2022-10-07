#![cfg(feature = "test-sbf")]

use solana_program_test::*;

mod program_test;

use program_test::*;
use spl_governance::state::{
    enums::{ProposalState, VoteThreshold},
    proposal::{OptionVoteResult, VoteType},
    vote_record::{Vote, VoteChoice},
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
