use std::collections::HashSet;

use crate::Command;

pub(crate) struct Client {
    transaction_queue: Option<Vec<Command>>,
    subscribed_channels: HashSet<String>,
}

impl Client {
    pub(crate) fn new() -> Self {
        Self {
            transaction_queue: None,
            subscribed_channels: HashSet::new(),
        }
    }

    // Transactions
    pub(crate) fn start_transactions(&mut self) {
        self.transaction_queue = Some(vec![]);
    }

    pub(crate) fn in_transaction(&mut self) -> bool {
        self.transaction_queue.is_some()
    }

    pub(crate) fn end_transaction(&mut self) -> Option<Vec<Command>> {
        self.transaction_queue.take()
    }

    pub(crate) fn enqueue(&mut self, cmd: Command) {
        if let Some(queue) = self.transaction_queue.as_mut() {
            queue.push(cmd);
        }
    }

    // PUB/SUB
    pub(crate) fn subscribe(&mut self, channel: String) {
        self.subscribed_channels.insert(channel);
    }

    pub(crate) fn in_subscribed_mode(&self) -> bool {
        !self.subscribed_channels.is_empty()
    }

    pub(crate) fn unsubscribe(&mut self) {
        self.subscribed_channels.clear();
    }

    pub(crate) fn subscriber_count(&mut self) -> usize {
        self.subscribed_channels.len()
    }
}
