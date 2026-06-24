----------------------------- MODULE AuthSpec -----------------------------
EXTENDS Naturals, Sequences, TLC

\* TLA+ Formal Specification for ChainLogistics Authorization System
\* This specification models the critical threshold checks and authorization invariants

CONSTANTS Signers, MaxSigners, DefaultThreshold, TimeLockDuration

\* Type definitions
Signer == Signers
ProposalId == Nat
Threshold == Nat
Timestamp == Nat

\* State variables
VARIABLES state, proposals, config, current_time

\* State definition
\* state = "uninitialized" | "configured"
\* config = [signers : Set(Signer), threshold : Threshold, thresholds : [kind -> Threshold], time_locks : [kind -> Nat]]
\* proposals = [ProposalId -> [kind : STRING, status : STRING, approvals : Set(Signer), rejections : Set(Signer), created_at : Timestamp, approved_at : Timestamp]]

\* Helper predicates
IsValidThreshold(t, n) == t > 0 /\ t <= n
IsValidSignerSet(S) == Cardinality(S) > 0 /\ Cardinality(S) <= MaxSigners
NoDuplicates(S) == Cardinality(S) = Cardinality(DOMAIN [s \in S |-> TRUE])

\* Invariant 1: Threshold validity
INV1 == state = "configured" => IsValidThreshold(config.threshold, Cardinality(config.signers))

\* Invariant 2: Signer set validity
INV2 == state = "configured" => IsValidSignerSet(config.signers) /\ NoDuplicates(config.signers)

\* Invariant 3: Proposal consistency
INV3 == \A p \in DOMAIN proposals :
    LET prop = proposals[p] IN
    /\ prop.approvals \subseteq config.signers
    /\ prop.rejections \subseteq config.signers
    /\ prop.approvals \cap prop.rejections = {}

\* Invariant 4: Threshold enforcement
INV4 == \A p \in DOMAIN proposals :
    LET prop = proposals[p] IN
    /\ (prop.status = "approved" => Cardinality(prop.approvals) >= GetThreshold(prop.kind))
    /\ (prop.status = "rejected" => Cardinality(prop.rejections) >= GetMaxRejections(prop.kind))

\* Invariant 5: Time lock enforcement
INV5 == \A p \in DOMAIN proposals :
    LET prop = proposals[p] IN
    /\ (prop.status = "executed" => current_time >= prop.approved_at + GetTimeLock(prop.kind))

\* Helper functions
GetThreshold(kind) == 
    IF kind \in DOMAIN config.thresholds 
    THEN config.thresholds[kind] 
    ELSE config.threshold

GetMaxRejections(kind) ==
    LET t == GetThreshold(kind) IN
    Cardinality(config.signers) - t + 1

GetTimeLock(kind) ==
    IF kind \in DOMAIN config.time_locks
    THEN config.time_locks[kind]
    ELSE 0

\* Initial state
Init ==
    /\ state = "uninitialized"
    /\ proposals = [p \in {1} |-> [kind |-> "", status |-> "", approvals |-> {}, rejections |-> {}, created_at |-> 0, approved_at |-> 0]]
    /\ config = [signers |-> {}, threshold |-> 0, thresholds |-> [kind \in {} |-> 0], time_locks |-> [kind \in {} |-> 0]]
    /\ current_time = 0

\* Actions
InitMultisig(signers, threshold, thresholds, time_locks) ==
    /\ state = "uninitialized"
    /\ IsValidSignerSet(signers)
    /\ NoDuplicates(signers)
    /\ IsValidThreshold(threshold, Cardinality(signers))
    /\ state' = "configured"
    /\ config' = [signers |-> signers, threshold |-> threshold, thresholds |-> thresholds, time_locks |-> time_locks]
    /\ UNCHANGED <<proposals, current_time>>

SubmitProposal(proposer, kind, args) ==
    /\ state = "configured"
    /\ proposer \in config.signers
    /\ LET new_id == Max(DOMAIN proposals) + 1 IN
       /\ proposals' = [proposals EXCEPT ![new_id] = [kind |-> kind, status |-> "active", approvals |-> {proposer}, rejections |-> {}, created_at |-> current_time, approved_at |-> 0]]
    /\ UNCHANGED <<state, config, current_time>>

ApproveProposal(approver, proposal_id) ==
    /\ state = "configured"
    /\ proposal_id \in DOMAIN proposals
    /\ approver \in config.signers
    /\ approver \notin proposals[proposal_id].approvals
    /\ approver \notin proposals[proposal_id].rejections
    /\ proposals[proposal_id].status = "active"
    /\ LET new_approvals == proposals[proposal_id].approvals \cup {approver} IN
       /\ LET new_status == IF Cardinality(new_approvals) >= GetThreshold(proposals[proposal_id].kind) THEN "approved" ELSE "active" IN
       /\ LET new_approved_at == IF new_status = "approved" THEN current_time ELSE proposals[proposal_id].approved_at IN
       /\ proposals' = [proposals EXCEPT ![proposal_id].approvals = new_approvals, ![proposal_id].status = new_status, ![proposal_id].approved_at = new_approved_at]
    /\ UNCHANGED <<state, config, current_time>>

RejectProposal(rejecter, proposal_id) ==
    /\ state = "configured"
    /\ proposal_id \in DOMAIN proposals
    /\ rejecter \in config.signers
    /\ rejecter \notin proposals[proposal_id].approvals
    /\ rejecter \notin proposals[proposal_id].rejections
    /\ proposals[proposal_id].status = "active"
    /\ LET new_rejections == proposals[proposal_id].rejections \cup {rejecter} IN
       /\ LET new_status == IF Cardinality(new_rejections) >= GetMaxRejections(proposals[proposal_id].kind) THEN "rejected" ELSE "active" IN
       /\ proposals' = [proposals EXCEPT ![proposal_id].rejections = new_rejections, ![proposal_id].status = new_status]
    /\ UNCHANGED <<state, config, current_time>>

ExecuteProposal(executor, proposal_id) ==
    /\ state = "configured"
    /\ proposal_id \in DOMAIN proposals
    /\ executor \in config.signers
    /\ proposals[proposal_id].status = "approved"
    /\ current_time >= proposals[proposal_id].approved_at + GetTimeLock(proposals[proposal_id].kind)
    /\ proposals' = [proposals EXCEPT ![proposal_id].status = "executed"]
    /\ UNCHANGED <<state, config, current_time>>

AdvanceTime(delta) ==
    /\ current_time' = current_time + delta
    /\ UNCHANGED <<state, proposals, config>>

\* Next state relation
Next ==
    \/ \E signers, threshold, thresholds, time_locks : InitMultisig(signers, threshold, thresholds, time_locks)
    \/ \E proposer, kind, args : SubmitProposal(proposer, kind, args)
    \/ \E approver, proposal_id : ApproveProposal(approver, proposal_id)
    \/ \E rejecter, proposal_id : RejectProposal(rejecter, proposal_id)
    \/ \E executor, proposal_id : ExecuteProposal(executor, proposal_id)
    \/ \E delta \in Nat : AdvanceTime(delta)

\* Type correctness invariant
TypeOK ==
    /\ state \in {"uninitialized", "configured"}
    /\ config.signers \subseteq Signers
    /\ config.threshold \in Threshold
    /\ \A p \in DOMAIN proposals : proposals[p].status \in {"active", "approved", "rejected", "executed"}

\* Theorem: Invariants hold
THEOREM Spec => [](INV1 /\ INV2 /\ INV3 /\ INV4 /\ INV5 /\ TypeOK)

\* Specification
Spec == Init /\ [][Next]_<<state, proposals, config, current_time>>

=============================================================================
