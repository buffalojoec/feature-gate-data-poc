use solana_program::{entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey};

pub struct StagedFeatures {
    current_epoch: u64,
    current_features: [Pubkey; 8],
    next_epoch: u64,
    next_features: [Pubkey; 8],
}

impl StagedFeatures {
    pub fn maybe_update(&mut self, current_epoch: u64) {
        if current_epoch >= self.next_epoch {
            self.current_epoch = self.next_epoch;
            self.current_features = self.next_features;
            self.next_epoch = current_epoch + 1;
            self.next_features = [Pubkey::default(); 8];
        }
    }

    pub fn stage_feature(
        &mut self,
        target_epoch: u64,
        feature_id: Pubkey,
    ) -> Result<(), ProgramError> {
        if target_epoch != self.next_epoch {
            // Target epoch is not the next epoch.
            return Err(ProgramError::InvalidArgument);
        }
        for feature in self.next_features.iter_mut() {
            if *feature == Pubkey::default() {
                *feature = feature_id;
                return Ok(());
            }
        }
        // Staged features is full.
        Err(ProgramError::InvalidArgument)
    }
}

pub struct SimulatedProgramContext {
    // This value could just be pulled from the `Clock` sysvar.
    clock_sysvar_epoch: u64,
    // The Staged Features PDA.
    staged_features_pda: StagedFeatures,
}

pub fn simulate_stage_feature_instruction(
    context: &mut SimulatedProgramContext,
    target_epoch: u64,
    feature_id: Pubkey,
) -> ProgramResult {
    /* Checks */
    context
        .staged_features_pda
        .maybe_update(context.clock_sysvar_epoch);
    context
        .staged_features_pda
        .stage_feature(target_epoch, feature_id)?;
    Ok(())
}

pub fn simulate_signal_support_instruction(
    context: &mut SimulatedProgramContext,
    _bitmask: u8,
) -> ProgramResult {
    /* Checks */
    context
        .staged_features_pda
        .maybe_update(context.clock_sysvar_epoch);
    /* Signal support logic... */
    Ok(())
}

#[test]
fn test() {
    // Set up the simulated program context.
    let mut context = SimulatedProgramContext {
        clock_sysvar_epoch: 0,
        staged_features_pda: StagedFeatures {
            current_epoch: 0,
            current_features: [Pubkey::default(); 8],
            next_epoch: 1,
            next_features: [Pubkey::default(); 8],
        },
    };

    // Fail trying to stage a feature for epoch 2.
    assert_eq!(
        simulate_stage_feature_instruction(&mut context, 2, Pubkey::new_unique()),
        Err(ProgramError::InvalidArgument)
    );

    // Stage a few features.
    let mock_feature_id_epoch_1 = Pubkey::new_unique();
    simulate_stage_feature_instruction(&mut context, 1, mock_feature_id_epoch_1).unwrap();
    simulate_stage_feature_instruction(&mut context, 1, mock_feature_id_epoch_1).unwrap();
    simulate_stage_feature_instruction(&mut context, 1, mock_feature_id_epoch_1).unwrap();

    // Current features are unchanged.
    assert_eq!(context.staged_features_pda.current_epoch, 0);
    assert_eq!(
        &context.staged_features_pda.current_features,
        &[
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::default(),
        ]
    );

    // Next epoch's features are staged.
    assert_eq!(context.staged_features_pda.next_epoch, 1);
    assert_eq!(
        &context.staged_features_pda.next_features,
        &[
            mock_feature_id_epoch_1,
            mock_feature_id_epoch_1,
            mock_feature_id_epoch_1,
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::default(),
        ]
    );

    // Move the epoch forward to 1.
    context.clock_sysvar_epoch = 1;

    // Fail trying to stage a feature for epoch 3.
    assert_eq!(
        simulate_stage_feature_instruction(&mut context, 3, Pubkey::new_unique()),
        Err(ProgramError::InvalidArgument)
    );

    // Succeed trying to stage a few features for epoch 2.
    let mock_feature_id_epoch_2 = Pubkey::new_unique();
    simulate_stage_feature_instruction(&mut context, 2, mock_feature_id_epoch_2).unwrap();
    simulate_stage_feature_instruction(&mut context, 2, mock_feature_id_epoch_2).unwrap();
    simulate_stage_feature_instruction(&mut context, 2, mock_feature_id_epoch_2).unwrap();
    simulate_stage_feature_instruction(&mut context, 2, mock_feature_id_epoch_2).unwrap();
    simulate_stage_feature_instruction(&mut context, 2, mock_feature_id_epoch_2).unwrap();

    // Current features are from epoch 1.
    assert_eq!(context.staged_features_pda.current_epoch, 1);
    assert_eq!(
        &context.staged_features_pda.current_features,
        &[
            mock_feature_id_epoch_1,
            mock_feature_id_epoch_1,
            mock_feature_id_epoch_1,
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::default(),
        ]
    );

    // Next epoch's features are staged, and for epoch 2.
    assert_eq!(context.staged_features_pda.next_epoch, 2);
    assert_eq!(
        &context.staged_features_pda.next_features,
        &[
            mock_feature_id_epoch_2,
            mock_feature_id_epoch_2,
            mock_feature_id_epoch_2,
            mock_feature_id_epoch_2,
            mock_feature_id_epoch_2,
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::default(),
        ]
    );

    // Move the epoch forward to 2.
    context.clock_sysvar_epoch = 2;

    // Simulate the first validator sending a support signal for epoch 2.
    simulate_signal_support_instruction(&mut context, 0b00000001).unwrap();

    // Current features are from epoch 2.
    assert_eq!(context.staged_features_pda.current_epoch, 2);
    assert_eq!(
        &context.staged_features_pda.current_features,
        &[
            mock_feature_id_epoch_2,
            mock_feature_id_epoch_2,
            mock_feature_id_epoch_2,
            mock_feature_id_epoch_2,
            mock_feature_id_epoch_2,
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::default(),
        ]
    );
}
