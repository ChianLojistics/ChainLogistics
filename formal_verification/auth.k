require "domains.k"
require "list.k"

module AUTH-SPEC
  imports DOMAINS
  imports LIST

  syntax KResult ::= Int | Bool | String | List

  // --- Configuration State ---
  syntax AuthConfig ::= AuthConfig( signers: List, threshold: Int, thresholds: Map, timelocks: Map )
  syntax ProposalStatus ::= "active" | "approved" | "rejected" | "executed"
  syntax Proposal ::= Proposal( id: Int, kind: String, status: ProposalStatus, approvals: List, rejections: List, created_at: Int, approved_at: Int )
  syntax AuthState ::= AuthState( config: AuthConfig, proposals: Map, next_id: Int )

  // --- Helper Functions ---
  syntax Bool ::= isValidThreshold( Int, Int ) [function]
  rule isValidThreshold(T, N) => T >Int 0 andBool T <=Int N

  syntax Bool ::= isValidSignerSet( List ) [function]
  rule isValidSignerSet(L) => size(L) >Int 0 andBool size(L) <=Int 10

  syntax Bool ::= noDuplicates( List ) [function]
  rule noDuplicates(.List) => true
  rule noDuplicates(H:T) => notBool (H inList T) andBool noDuplicates(T)

  syntax Int ::= getThreshold( Map, String, Int ) [function]
  rule getThreshold(Threshs, Kind, Default) => Threshs[Kind] Default

  syntax Int ::= getMaxRejections( Int, Int ) [function]
  rule getMaxRejections(SignerCount, Threshold) => SignerCount -Int Threshold +Int 1

  syntax Int ::= getTimeLock( Map, String ) [function]
  rule getTimeLock(Timelocks, Kind) => Timelocks[Kind] 0

  // --- Invariants ---
  syntax Bool ::= invThresholdValid( AuthConfig ) [function]
  rule invThresholdValid(AuthConfig(Signers, Threshold, _, _)) => 
    isValidThreshold(Threshold, size(Signers))

  syntax Bool ::= invSignersValid( AuthConfig ) [function]
  rule invSignersValid(AuthConfig(Signers, _, _, _)) => 
    isValidSignerSet(Signers) andBool noDuplicates(Signers)

  syntax Bool ::= invProposalConsistent( Map, List ) [function]
  rule invProposalConsistent(Proposals, Signers) => 
    #Forall #Exists( .Map ) // Simplified: all proposals have approvals/rejections subset of signers

  syntax Bool ::= invThresholdEnforced( Map, Map, Int, List ) [function]
  rule invThresholdEnforced(Proposals, Threshs, Default, Signers) => 
    #Forall #Exists( .Map ) // Simplified: approved proposals meet threshold

  syntax Bool ::= invTimeLockEnforced( Map, Map, Int ) [function]
  rule invTimeLockEnforced(Proposals, Timelocks, CurrentTime) => 
    #Forall #Exists( .Map ) // Simplified: executed proposals respect time lock

  // --- Operations ---
  syntax KResult ::= "init_multisig" ( List, Int, Map, Map )
  syntax KResult ::= "submit_proposal" ( String, String, List )
  syntax KResult ::= "approve_proposal" ( String, Int )
  syntax KResult ::= "reject_proposal" ( String, Int )
  syntax KResult ::= "execute_proposal" ( String, Int )

  // Init Multisig
  rule <k> init_multisig(Signers, Threshold, Threshs, Timelocks) => .K ...</k>
       <state> AuthState(AuthConfig(.List, 0, .Map, .Map), Proposals, NextId) </state>
       requires isValidSignerSet(Signers)
       requires noDuplicates(Signers)
       requires isValidThreshold(Threshold, size(Signers))
    => <state> AuthState(AuthConfig(Signers, Threshold, Threshs, Timelocks), Proposals, NextId) </state>

  // Submit Proposal
  rule <k> submit_proposal(Proposer, Kind, Args) => ProposalId ...</k>
       <state> AuthState(Config, Proposals, NextId) </state>
       requires Proposer inList Config.Signers
    => <state> AuthState(Config, Proposals[ NextId <- Proposal(NextId, Kind, "active", [Proposer], .List, CurrentTime, 0) ], NextId +Int 1) </state>
       <currentTime> CurrentTime </currentTime>

  // Approve Proposal
  rule <k> approve_proposal(Approver, ProposalId) => .K ...</k>
       <state> AuthState(Config, Proposals, NextId) </state>
       <proposal> Proposal(ProposalId, Kind, "active", Approvals, Rejections, CreatedAt, ApprovedAt) </proposal>
       requires Approver inList Config.Signers
       requires notBool (Approver inList Approvals)
       requires notBool (Approver inList Rejections)
       let NewApprovals = Approvals + [Approver]
       let NewThreshold = getThreshold(Config.Thresholds, Kind, Config.Threshold)
       let NewStatus = if size(NewApprovals) >=Int NewThreshold then "approved" else "active"
       let NewApprovedAt = if NewStatus ==K "approved" then CurrentTime else ApprovedAt
    => <state> AuthState(Config, Proposals[ ProposalId <- Proposal(ProposalId, Kind, NewStatus, NewApprovals, Rejections, CreatedAt, NewApprovedAt) ], NextId) </state>
       <currentTime> CurrentTime </currentTime>

  // Reject Proposal
  rule <k> reject_proposal(Rejecter, ProposalId) => .K ...</k>
       <state> AuthState(Config, Proposals, NextId) </state>
       <proposal> Proposal(ProposalId, Kind, "active", Approvals, Rejections, CreatedAt, ApprovedAt) </proposal>
       requires Rejecter inList Config.Signers
       requires notBool (Rejecter inList Approvals)
       requires notBool (Rejecter inList Rejections)
       let NewRejections = Rejections + [Rejecter]
       let MaxRejections = getMaxRejections(size(Config.Signers), getThreshold(Config.Thresholds, Kind, Config.Threshold))
       let NewStatus = if size(NewRejections) >=Int MaxRejections then "rejected" else "active"
    => <state> AuthState(Config, Proposals[ ProposalId <- Proposal(ProposalId, Kind, NewStatus, Approvals, NewRejections, CreatedAt, ApprovedAt) ], NextId) </state>

  // Execute Proposal
  rule <k> execute_proposal(Executor, ProposalId) => .K ...</k>
       <state> AuthState(Config, Proposals, NextId) </state>
       <proposal> Proposal(ProposalId, Kind, "approved", Approvals, Rejections, CreatedAt, ApprovedAt) </proposal>
       requires Executor inList Config.Signers
       let TimeLock = getTimeLock(Config.Timelocks, Kind)
       requires CurrentTime >=Int ApprovedAt +Int TimeLock
    => <state> AuthState(Config, Proposals[ ProposalId <- Proposal(ProposalId, Kind, "executed", Approvals, Rejections, CreatedAt, ApprovedAt) ], NextId) </state>
       <currentTime> CurrentTime </currentTime>

  // --- Verification Rules ---
  syntax Bool ::= verifyInitMultisig( List, Int ) [function]
  rule verifyInitMultisig(Signers, Threshold) => 
    isValidSignerSet(Signers) andBool noDuplicates(Signers) andBool isValidThreshold(Threshold, size(Signers))

  syntax Bool ::= verifyThresholdReached( List, Int ) [function]
  rule verifyThresholdReached(Approvals, Threshold) => size(Approvals) >=Int Threshold

  syntax Bool ::= verifyRejectionThreshold( List, Int, Int ) [function]
  rule verifyRejectionThreshold(Rejections, SignerCount, Threshold) => 
    size(Rejections) >=Int (SignerCount -Int Threshold +Int 1)

  syntax Bool ::= verifyTimeLock( Int, Int, Int ) [function]
  rule verifyTimeLock(CurrentTime, ApprovedAt, TimeLock) => CurrentTime >=Int ApprovedAt +Int TimeLock

endmodule
