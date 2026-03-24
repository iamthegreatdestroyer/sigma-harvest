//! Priority claim queue with retry logic.

use super::ClaimOperation;
use std::collections::BinaryHeap;

/// A priority queue for claim operations, ordered by harvest score.
pub struct ClaimQueue {
    queue: BinaryHeap<PrioritizedClaim>,
}

struct PrioritizedClaim {
    priority: u32,
    operation: ClaimOperation,
}

impl Ord for PrioritizedClaim {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority.cmp(&other.priority)
    }
}
impl PartialOrd for PrioritizedClaim {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for PrioritizedClaim {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}
impl Eq for PrioritizedClaim {}

impl ClaimQueue {
    pub fn new() -> Self {
        Self {
            queue: BinaryHeap::new(),
        }
    }

    pub fn enqueue(&mut self, operation: ClaimOperation, priority: u32) {
        self.queue.push(PrioritizedClaim { priority, operation });
    }

    pub fn dequeue(&mut self) -> Option<ClaimOperation> {
        self.queue.pop().map(|p| p.operation)
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::{ClaimOperation, ClaimStatus};

    fn make_claim(id: &str) -> ClaimOperation {
        ClaimOperation {
            id: id.to_string(),
            opportunity_id: format!("opp-{}", id),
            wallet_address: "0xABC".to_string(),
            chain: "ethereum".to_string(),
            contract_address: Some("0xDef".to_string()),
            calldata: Some("0xa9059cbb".to_string()),
            status: ClaimStatus::Pending,
            gas_limit: Some(21000),
            retry_count: 0,
            max_retries: 3,
            harvest_score: 50,
        }
    }

    #[test]
    fn new_queue_is_empty() {
        let q = ClaimQueue::new();
        assert!(q.is_empty());
        assert_eq!(q.len(), 0);
    }

    #[test]
    fn enqueue_increases_length() {
        let mut q = ClaimQueue::new();
        q.enqueue(make_claim("1"), 50);
        assert_eq!(q.len(), 1);
        assert!(!q.is_empty());
    }

    #[test]
    fn dequeue_returns_highest_priority() {
        let mut q = ClaimQueue::new();
        q.enqueue(make_claim("low"), 10);
        q.enqueue(make_claim("high"), 90);
        q.enqueue(make_claim("mid"), 50);
        let top = q.dequeue().unwrap();
        assert_eq!(top.id, "high");
    }

    #[test]
    fn dequeue_order_is_descending_priority() {
        let mut q = ClaimQueue::new();
        q.enqueue(make_claim("a"), 30);
        q.enqueue(make_claim("b"), 80);
        q.enqueue(make_claim("c"), 50);
        assert_eq!(q.dequeue().unwrap().id, "b");
        assert_eq!(q.dequeue().unwrap().id, "c");
        assert_eq!(q.dequeue().unwrap().id, "a");
    }

    #[test]
    fn dequeue_empty_returns_none() {
        let mut q = ClaimQueue::new();
        assert!(q.dequeue().is_none());
    }

    #[test]
    fn enqueue_multiple_same_priority() {
        let mut q = ClaimQueue::new();
        q.enqueue(make_claim("a"), 50);
        q.enqueue(make_claim("b"), 50);
        assert_eq!(q.len(), 2);
        // Both should dequeue without panic
        q.dequeue().unwrap();
        q.dequeue().unwrap();
        assert!(q.is_empty());
    }

    #[test]
    fn enqueue_100_items() {
        let mut q = ClaimQueue::new();
        for i in 0..100 {
            q.enqueue(make_claim(&i.to_string()), i as u32);
        }
        assert_eq!(q.len(), 100);
        // Highest priority should be dequeued first
        let top = q.dequeue().unwrap();
        assert_eq!(top.id, "99");
    }
}
