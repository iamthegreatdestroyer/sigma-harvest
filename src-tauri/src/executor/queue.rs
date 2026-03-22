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
