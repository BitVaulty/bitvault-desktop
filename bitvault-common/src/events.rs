use crossbeam_channel::{unbounded, Sender, Receiver};
use std::thread;
use std::sync::{Arc, Mutex};
use std::collections::{HashMap, VecDeque};
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;
use crate::logging::{log_security, LogLevel};
use serde_json::json;
use serde::{Serialize, Deserialize};
use std::fmt;
use std::time::{Duration, Instant};
use std::collections::hash_map::Entry;

/// MessagePriority defines the urgency level of wallet events
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessagePriority {
    Low,
    Medium,
    High,
    Critical
}

/// EventType categorizes the different types of wallet events
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventType {
    WalletUpdate,
    TransactionReceived,
    TransactionSent,
    TransactionConfirmed,
    NetworkStatus,
    SecurityAlert,
    BackupRequired,
    SyncStatus,
    Settings,
    System,
    // Security boundary event types
    CoreRequest,  // Request crossing security boundary to core
    CoreResponse, // Response crossing security boundary from core
    UiRequest,    // Request from UI to less secure components
    UiResponse,   // Response to UI from less secure components
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventType::WalletUpdate => write!(f, "WalletUpdate"),
            EventType::TransactionReceived => write!(f, "TransactionReceived"),
            EventType::TransactionSent => write!(f, "TransactionSent"),
            EventType::TransactionConfirmed => write!(f, "TransactionConfirmed"),
            EventType::NetworkStatus => write!(f, "NetworkStatus"),
            EventType::SecurityAlert => write!(f, "SecurityAlert"),
            EventType::BackupRequired => write!(f, "BackupRequired"),
            EventType::SyncStatus => write!(f, "SyncStatus"),
            EventType::Settings => write!(f, "Settings"),
            EventType::System => write!(f, "System"),
            EventType::CoreRequest => write!(f, "CoreRequest"),
            EventType::CoreResponse => write!(f, "CoreResponse"),
            EventType::UiRequest => write!(f, "UiRequest"),
            EventType::UiResponse => write!(f, "UiResponse"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IpcMessage {
    pub id: u64,
    pub event_type: EventType,
    pub payload: String,
    pub timestamp: String,
    pub priority: MessagePriority,
}

impl IpcMessage {
    pub fn new(id: u64, event_type: EventType, payload: &str, priority: MessagePriority) -> Self {
        Self {
            id,
            event_type,
            payload: payload.to_string(),
            timestamp: chrono::Local::now().to_rfc3339(),
            priority,
        }
    }
    
    /// Create a security alert message with Critical priority
    pub fn security_alert(id: u64, payload: &str) -> Self {
        Self::new(id, EventType::SecurityAlert, payload, MessagePriority::Critical)
    }
    
    /// Create a system message with Low priority
    pub fn system(id: u64, payload: &str) -> Self {
        Self::new(id, EventType::System, payload, MessagePriority::Low)
    }
    
    /// Create a transaction notification
    pub fn transaction(id: u64, payload: &str, is_received: bool) -> Self {
        let event_type = if is_received {
            EventType::TransactionReceived
        } else {
            EventType::TransactionSent
        };
        Self::new(id, event_type, payload, MessagePriority::High)
    }

    /// Create a core request message
    pub fn core_request(id: u64, payload: &str) -> Self {
        Self::new(id, EventType::CoreRequest, payload, MessagePriority::Medium)
    }

    /// Create a core response message
    pub fn core_response(id: u64, payload: &str) -> Self {
        Self::new(id, EventType::CoreResponse, payload, MessagePriority::Medium)
    }
}

/// Maximum number of events to keep in memory for replay
const MAX_EVENT_HISTORY: usize = 1000;

/// EventStorage handles persisting critical events to disk
pub struct EventStorage {
    event_history: Arc<Mutex<VecDeque<IpcMessage>>>,
    storage_path: Option<String>,
}

impl EventStorage {
    pub fn new(storage_path: Option<String>) -> Self {
        let mut event_history = VecDeque::with_capacity(MAX_EVENT_HISTORY);
        
        // Try to load persisted events if path is provided
        if let Some(ref path) = storage_path {
            if let Err(e) = Self::load_persisted_events(path, &mut event_history) {
                log_security(
                    LogLevel::Error,
                    "Failed to load persisted events",
                    Some(json!({"error": e.to_string(), "path": path}))
                );
            }
        }
        
        Self {
            event_history: Arc::new(Mutex::new(event_history)),
            storage_path,
        }
    }
    
    /// Save a critical event to persistent storage
    pub fn persist_event(&self, message: &IpcMessage) -> io::Result<()> {
        // Only persist critical events and security alerts
        if message.priority != MessagePriority::Critical && 
           message.event_type != EventType::SecurityAlert {
            return Ok(());
        }
        
        // Add to in-memory history
        {
            let mut history = self.event_history.lock().unwrap();
            if history.len() >= MAX_EVENT_HISTORY {
                history.pop_front();
            }
            history.push_back(message.clone());
        }
        
        // Save to disk if storage path is set
        if let Some(ref path) = self.storage_path {
            let json = serde_json::to_string(message)?;
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)?;
                
            writeln!(file, "{}", json)?;
        }
        
        Ok(())
    }
    
    /// Get a copy of the event history
    pub fn get_event_history(&self) -> Vec<IpcMessage> {
        let history = self.event_history.lock().unwrap();
        history.iter().cloned().collect()
    }
    
    /// Load persisted events from file
    fn load_persisted_events(path: &str, history: &mut VecDeque<IpcMessage>) -> io::Result<()> {
        if !Path::new(path).exists() {
            return Ok(());
        }
        
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        
        for line in contents.lines() {
            match serde_json::from_str::<IpcMessage>(line) {
                Ok(event) => {
                    if history.len() >= MAX_EVENT_HISTORY {
                        history.pop_front();
                    }
                    history.push_back(event);
                }
                Err(e) => {
                    log_security(
                        LogLevel::Error,
                        "Failed to parse persisted event",
                        Some(json!({"error": e.to_string(), "line": line}))
                    );
                }
            }
        }
        
        Ok(())
    }
}

/// Rate limit configuration
#[derive(Clone, Copy, Debug)]
pub struct RateLimitConfig {
    /// Maximum number of events allowed in the specified time window
    pub max_events: usize,
    /// Time window for rate limiting in milliseconds
    pub time_window_ms: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_events: 100,
            time_window_ms: 1000, // 1 second
        }
    }
}

/// Rate limiter for controlling event flow
pub struct RateLimiter {
    config: RateLimitConfig,
    event_counts: HashMap<EventType, Vec<Instant>>,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            event_counts: HashMap::new(),
        }
    }
    
    /// Check if an event should be rate limited
    pub fn should_limit(&mut self, event_type: EventType) -> bool {
        // Critical events are never rate-limited
        if event_type == EventType::SecurityAlert {
            return false;
        }
        
        let now = Instant::now();
        let window = Duration::from_millis(self.config.time_window_ms);
        
        match self.event_counts.entry(event_type) {
            Entry::Vacant(entry) => {
                entry.insert(vec![now]);
                false
            }
            Entry::Occupied(mut entry) => {
                let timestamps = entry.get_mut();
                
                // Remove timestamps outside the window
                let cutoff = now.checked_sub(window).unwrap_or(now);
                timestamps.retain(|&time| time >= cutoff);
                
                // Check if we're over the limit
                if timestamps.len() >= self.config.max_events {
                    true
                } else {
                    timestamps.push(now);
                    false
                }
            }
        }
    }
}

/// EventFilter defines criteria for which events should be processed
pub struct EventFilter {
    allowed_types: Option<Vec<EventType>>,
    min_priority: Option<MessagePriority>,
    custom_filter: Option<Box<dyn Fn(&IpcMessage) -> bool + Send + Sync>>,
}

impl EventFilter {
    pub fn new() -> Self {
        Self {
            allowed_types: None,
            min_priority: None,
            custom_filter: None,
        }
    }
    
    /// Only allow specific event types
    pub fn with_event_types(mut self, types: Vec<EventType>) -> Self {
        self.allowed_types = Some(types);
        self
    }
    
    /// Only allow events with at least the specified priority
    pub fn with_min_priority(mut self, priority: MessagePriority) -> Self {
        self.min_priority = Some(priority);
        self
    }
    
    /// Add a custom filter function
    pub fn with_custom_filter<F>(mut self, filter: F) -> Self 
    where 
        F: Fn(&IpcMessage) -> bool + Send + Sync + 'static 
    {
        self.custom_filter = Some(Box::new(filter));
        self
    }
    
    /// Check if an event should be processed based on the filter criteria
    pub fn should_process(&self, message: &IpcMessage) -> bool {
        // Check event type if filter is set
        if let Some(ref allowed) = self.allowed_types {
            if !allowed.contains(&message.event_type) {
                return false;
            }
        }
        
        // Check priority if filter is set
        if let Some(min_priority) = self.min_priority {
            // Compare priorities - this assumes an ordering of Low < Medium < High < Critical
            let meets_priority = match (min_priority, message.priority) {
                (MessagePriority::Low, _) => true,
                (MessagePriority::Medium, MessagePriority::Low) => false,
                (MessagePriority::Medium, _) => true,
                (MessagePriority::High, MessagePriority::Critical) => true,
                (MessagePriority::High, MessagePriority::High) => true,
                (MessagePriority::High, _) => false,
                (MessagePriority::Critical, MessagePriority::Critical) => true,
                (MessagePriority::Critical, _) => false,
            };
            
            if !meets_priority {
                return false;
            }
        }
        
        // Apply custom filter if set
        if let Some(ref filter) = self.custom_filter {
            if !filter(message) {
                return false;
            }
        }
        
        true
    }
}

impl Default for EventFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// A thread-safe event dispatcher that handles IPC messages across components
pub struct EventDispatcher {
    sender: Sender<IpcMessage>,
    pub receiver: Receiver<IpcMessage>,
    next_id: Arc<Mutex<u64>>,
    subscribers: Arc<Mutex<HashMap<EventType, Vec<Sender<IpcMessage>>>>>,
    is_running: Arc<Mutex<bool>>,
    storage: EventStorage,
}

impl EventDispatcher {
    pub fn new() -> Self {
        Self::with_storage(None)
    }
    
    pub fn with_storage(storage_path: Option<String>) -> Self {
        let (sender, receiver) = unbounded();
        Self { 
            sender, 
            receiver, 
            next_id: Arc::new(Mutex::new(1)),
            subscribers: Arc::new(Mutex::new(HashMap::new())),
            is_running: Arc::new(Mutex::new(false)),
            storage: EventStorage::new(storage_path),
        }
    }

    /// Get a unique message ID
    pub fn next_id(&self) -> u64 {
        let mut id = self.next_id.lock().unwrap();
        let current = *id;
        *id += 1;
        current
    }

    /// Dispatch a message to all subscribers
    pub fn dispatch(&self, message: IpcMessage) {
        log_security(
            LogLevel::Info,
            &format!("Dispatching message: {:?}", message),
            None
        );
        // Log all critical messages for security auditing
        if message.priority == MessagePriority::Critical {
            log_security(
                LogLevel::Warn, 
                &format!("Critical event: {}", message.event_type), 
                Some(json!({ "message": message.clone() }))
            );
        }

        // Persist the event if needed
        if let Err(e) = self.storage.persist_event(&message) {
            log_security(
                LogLevel::Error,
                "Failed to persist event",
                Some(json!({"error": e.to_string(), "event_type": message.event_type.to_string()}))
            );
        }

        // Send message to global channel
        if let Err(e) = self.sender.send(message.clone()) {
            log_security(
                LogLevel::Error, 
                "Failed to dispatch message to global channel", 
                Some(json!({ "error": e.to_string(), "message": message.clone() }))
            );
        }

        // Send to specific subscribers for this event type
        let subscribers = self.subscribers.lock().unwrap();
        if let Some(channels) = subscribers.get(&message.event_type) {
            for channel in channels {
                if let Err(e) = channel.send(message.clone()) {
                    log_security(
                        LogLevel::Error, 
                        "Failed to dispatch message to subscriber", 
                        Some(json!({ "error": e.to_string(), "event_type": message.event_type.to_string() }))
                    );
                }
            }
        }
    }

    /// Create a new message and dispatch it
    pub fn create_and_dispatch(&self, event_type: EventType, payload: &str, priority: MessagePriority) {
        let id = self.next_id();
        let message = IpcMessage::new(id, event_type, payload, priority);
        self.dispatch(message);
    }

    /// Subscribe to specific event types
    pub fn subscribe(&self, event_type: EventType) -> Receiver<IpcMessage> {
        let (sender, receiver) = unbounded();
        let mut subscribers = self.subscribers.lock().unwrap();
        
        subscribers.entry(event_type)
            .or_insert_with(Vec::new)
            .push(sender);
            
        log_security(
            LogLevel::Info, 
            &format!("New subscriber for event type: {}", event_type), 
            None
        );
        
        receiver
    }
    
    /// Start the event processing loop in a background thread
    pub fn start(&mut self) {
        // Create a separate clone of the receiver for the background thread
        // This is safer than using self.receiver.clone() directly
        let receiver_clone = self.receiver.clone();
        let is_running = Arc::clone(&self.is_running);
        
        // Set running state to true
        {
            let mut running = is_running.lock().unwrap();
            *running = true;
        }
        
        thread::spawn(move || {
            log_security(LogLevel::Info, "Event dispatcher thread started", None);
            let mut message_count = 0;
            
            while let Ok(message) = receiver_clone.recv() {
                message_count += 1;
                log_security(
                    LogLevel::Info,
                    &format!("Received message: {:?} (Total: {})", message, message_count),
                    None
                );
                // Check if we should stop the processing
                log_security(LogLevel::Info, "Checking running state", None);
                {
                    let running = is_running.lock().unwrap();
                    log_security(LogLevel::Info, &format!("is_running: {}", *running), None);
                    if !*running {
                        log_security(LogLevel::Info, "Stopping event dispatcher thread", None);
                        break;
                    }
                }
                
                // Simplified message processing
                log_security(
                    LogLevel::Info,
                    &format!("Processing event: {}", message.event_type),
                    None
                );
            }
            
            log_security(LogLevel::Info, "Event dispatcher thread stopped", None);
        });
    }
    
    /// Stop the event processing
    pub fn stop(&self) {
        let mut running = self.is_running.lock().unwrap();
        *running = false;
        log_security(LogLevel::Info, "Event dispatcher shutting down", None);
    }

    /// Get event history for replay
    pub fn get_event_history(&self) -> Vec<IpcMessage> {
        self.storage.get_event_history()
    }
    
    /// Subscribe and replay previous events of specified type
    pub fn subscribe_with_replay(&self, event_type: EventType) -> Receiver<IpcMessage> {
        let (sender, receiver) = unbounded();
        
        // Add to subscribers
        {
            let mut subscribers = self.subscribers.lock().unwrap();
            subscribers.entry(event_type)
                .or_insert_with(Vec::new)
                .push(sender.clone());
        }
        
        // Replay historical events of this type
        let history = self.storage.get_event_history();
        for event in history {
            if event.event_type == event_type {
                // Ignore errors - receiver might not be ready yet
                let _ = sender.send(event);
            }
        }
        
        receiver
    }
}

/// DeadLetterChannel collects messages that couldn't be processed successfully
pub struct DeadLetterChannel {
    sender: Sender<(IpcMessage, String)>,
    receiver: Receiver<(IpcMessage, String)>,
    max_capacity: usize,
    failed_messages: Arc<Mutex<VecDeque<(IpcMessage, String)>>>,
}

impl DeadLetterChannel {
    pub fn new(max_capacity: usize) -> Self {
        let (sender, receiver) = unbounded();
        Self {
            sender,
            receiver,
            max_capacity,
            failed_messages: Arc::new(Mutex::new(VecDeque::with_capacity(max_capacity))),
        }
    }
    
    pub fn get_sender(&self) -> Sender<(IpcMessage, String)> {
        self.sender.clone()
    }
    
    pub fn record_failure(&self, message: IpcMessage, reason: &str) {
        if let Err(e) = self.sender.send((message.clone(), reason.to_string())) {
            log_security(
                LogLevel::Error,
                "Failed to record message failure",
                Some(json!({
                    "error": e.to_string(),
                    "message_id": message.id
                }))
            );
        }
    }
    
    pub fn start_collecting(&self) {
        let receiver = self.receiver.clone();
        let failed_messages = Arc::clone(&self.failed_messages);
        let max_capacity = self.max_capacity;
        
        thread::spawn(move || {
            while let Ok((message, reason)) = receiver.recv() {
                let mut messages = failed_messages.lock().unwrap();
                
                // Keep within capacity
                if messages.len() >= max_capacity {
                    messages.pop_front();
                }
                
                // Log all dead letter events
                log_security(
                    LogLevel::Warn,
                    "Message processing failed",
                    Some(json!({
                        "message_id": message.id,
                        "event_type": message.event_type.to_string(),
                        "reason": reason,
                    }))
                );
                
                messages.push_back((message, reason));
            }
        });
    }
    
    pub fn get_failed_messages(&self) -> Vec<(IpcMessage, String)> {
        let messages = self.failed_messages.lock().unwrap();
        messages.iter().cloned().collect()
    }
}

/// Enhanced MessageBus with dead-letter channel
pub struct MessageBus {
    dispatcher: EventDispatcher,
    rate_limiter: Arc<Mutex<RateLimiter>>,
    dead_letter_channel: Option<DeadLetterChannel>,
}

impl MessageBus {
    pub fn new() -> Self {
        Self {
            dispatcher: EventDispatcher::new(),
            rate_limiter: Arc::new(Mutex::new(RateLimiter::new(RateLimitConfig::default()))),
            dead_letter_channel: None,
        }
    }
    
    pub fn with_config(
        storage_path: Option<String>, 
        rate_limit_config: RateLimitConfig,
        enable_dead_letter: bool,
    ) -> Self {
        let dead_letter = if enable_dead_letter {
            let dlc = DeadLetterChannel::new(100);
            dlc.start_collecting();
            Some(dlc)
        } else {
            None
        };
        
        Self {
            dispatcher: EventDispatcher::with_storage(storage_path),
            rate_limiter: Arc::new(Mutex::new(RateLimiter::new(rate_limit_config))),
            dead_letter_channel: dead_letter,
        }
    }
    
    pub fn start(&mut self) {
        // Call start on the dispatcher (modified to be safer)
        self.dispatcher.start();
        
        // Log startup
        log_security(
            LogLevel::Info,
            "MessageBus started",
            None
        );
    }
    
    pub fn stop(&self) {
        // Stop the event dispatcher
        self.dispatcher.stop();
        
        // Log shutdown
        log_security(
            LogLevel::Info,
            "MessageBus stopped",
            None
        );
    }
    
    // Rate-limited publish
    pub fn publish(&self, event_type: EventType, payload: &str, priority: MessagePriority) {
        // Skip rate limiting for critical events
        if priority != MessagePriority::Critical {
            let mut limiter = self.rate_limiter.lock().unwrap();
            if limiter.should_limit(event_type) {
                log_security(
                    LogLevel::Warn,
                    &format!("Rate limiting applied to event: {}", event_type),
                    Some(json!({ "event_type": event_type.to_string() }))
                );
                return;
            }
        }
        
        self.dispatcher.create_and_dispatch(event_type, payload, priority);
    }
    
    pub fn subscribe(&self, event_type: EventType) -> Receiver<IpcMessage> {
        self.dispatcher.subscribe(event_type)
    }
    
    // Subscribe with a filter
    pub fn subscribe_filtered(&self, filter: EventFilter) -> Receiver<IpcMessage> {
        let (sender, receiver) = unbounded();
        let global_receiver = self.dispatcher.receiver.clone();
        
        thread::spawn(move || {
            while let Ok(message) = global_receiver.recv() {
                if filter.should_process(&message) {
                    if sender.send(message).is_err() {
                        // Subscriber has been dropped, stop filtering
                        break;
                    }
                }
            }
        });
        
        receiver
    }
    
    // Subscribe with replay
    pub fn subscribe_with_replay(&self, event_type: EventType) -> Receiver<IpcMessage> {
        self.dispatcher.subscribe_with_replay(event_type)
    }
    
    pub fn subscribe_multiple(&self, event_types: &[EventType]) -> Receiver<IpcMessage> {
        // Create a new channel
        let (sender, receiver) = unbounded();
        
        // For each event type, create a subscription and spawn a thread to forward messages
        for &event_type in event_types {
            let event_receiver = self.dispatcher.subscribe(event_type);
            let event_sender = sender.clone();
            
            thread::spawn(move || {
                while let Ok(message) = event_receiver.recv() {
                    if event_sender.send(message).is_err() {
                        // Subscriber has been dropped, stop forwarding
                        break;
                    }
                }
            });
        }
        
        receiver
    }
    
    /// Report a failed message processing to the dead-letter channel
    pub fn report_failed_processing(&self, message: IpcMessage, reason: &str) {
        if let Some(ref dlc) = self.dead_letter_channel {
            dlc.record_failure(message, reason);
        }
    }
    
    pub fn get_failed_messages(&self) -> Vec<(IpcMessage, String)> {
        if let Some(ref dlc) = self.dead_letter_channel {
            dlc.get_failed_messages()
        } else {
            Vec::new()
        }
    }
}

/// Security testing helpers for event system - only compiled in test mode
#[cfg(test)]
pub mod test_helpers {
    use super::*;
    
    /// Create a test message with the given type
    pub fn create_test_message(event_type: EventType, priority: MessagePriority) -> IpcMessage {
        IpcMessage::new(
            1, 
            event_type, 
            &format!("Test message for {:?}", event_type),
            priority
        )
    }
    
    /// Verify an event's security properties
    pub fn verify_event_security(message: &IpcMessage) -> Result<(), &'static str> {
        // Check for security boundary crossing
        match message.event_type {
            EventType::CoreRequest | EventType::CoreResponse => {
                // These events cross security boundaries
                if message.priority != MessagePriority::High && 
                   message.priority != MessagePriority::Critical {
                    return Err("Security boundary events must have High or Critical priority");
                }
            }
            _ => {}
        }
        
        // Verify timestamp is present
        if message.timestamp.is_empty() {
            return Err("Event must have a timestamp");
        }
        
        Ok(())
    }
    
    /// Create a MessageBus configured for testing
    pub fn create_test_message_bus() -> MessageBus {
        let mut bus = MessageBus::with_config(
            None,
            RateLimitConfig {
                max_events: 10,
                time_window_ms: 100,
            },
            true
        );
        bus.start();
        bus
    }
    
    /// Assert that a receiver gets a message of the expected type
    pub fn assert_receives_event_type(
        receiver: &Receiver<IpcMessage>, 
        expected_type: EventType,
        timeout_ms: u64
    ) -> Result<IpcMessage, &'static str> {
        // Try to receive with timeout
        match receiver.recv_timeout(Duration::from_millis(timeout_ms)) {
            Ok(msg) => {
                assert_eq!(msg.event_type, expected_type, 
                    "Expected event type {:?}, got {:?}", expected_type, msg.event_type);
                Ok(msg)
            }
            Err(_) => Err("Timed out waiting for message")
        }
    }
}

/// Event types for key management operations
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum KeyManagementEvent {
    /// Triggered when a key is generated
    KeyGenerated,
    /// Triggered when a key is encrypted
    KeyEncrypted,
    /// Triggered when key decryption fails
    KeyDecryptionFailed,
}

/// Simple message bus for publishing and subscribing to key management events
/// 
/// For larger event needs, use the more comprehensive MessageBus implementation
/// in this module.
pub struct KeyManagementBus {
    subscribers: Arc<Mutex<HashMap<KeyManagementEvent, Vec<std::sync::mpsc::Sender<KeyManagementEvent>>>>>,
}

impl KeyManagementBus {
    /// Create a new key management bus
    pub fn new() -> Self {
        KeyManagementBus {
            subscribers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Subscribe to a key management event
    pub fn subscribe(&self, event: KeyManagementEvent) -> std::sync::mpsc::Receiver<KeyManagementEvent> {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut subscribers = self.subscribers.lock().unwrap();
        subscribers.entry(event.clone()).or_insert_with(Vec::new).push(tx);
        rx
    }

    /// Publish a key management event
    pub fn publish(&self, event: KeyManagementEvent) {
        if let Some(subscribers) = self.subscribers.lock().unwrap().get(&event) {
            for subscriber in subscribers {
                let _ = subscriber.send(event.clone());
            }
        }
    }
}

#[cfg(test)]
mod key_management_tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_event_subscription() {
        let bus = KeyManagementBus::new();
        let receiver = bus.subscribe(KeyManagementEvent::KeyGenerated);

        // Publish an event
        bus.publish(KeyManagementEvent::KeyGenerated);

        // Check if the event is received
        assert_eq!(receiver.recv().unwrap(), KeyManagementEvent::KeyGenerated);
    }

    #[test]
    fn test_event_publishing() {
        let bus = KeyManagementBus::new();
        let receiver1 = bus.subscribe(KeyManagementEvent::KeyEncrypted);
        let receiver2 = bus.subscribe(KeyManagementEvent::KeyEncrypted);

        // Publish an event
        bus.publish(KeyManagementEvent::KeyEncrypted);

        // Check if both subscribers receive the event
        assert_eq!(receiver1.recv().unwrap(), KeyManagementEvent::KeyEncrypted);
        assert_eq!(receiver2.recv().unwrap(), KeyManagementEvent::KeyEncrypted);
    }

    #[test]
    fn test_no_event_received_for_different_subscription() {
        let bus = KeyManagementBus::new();
        let receiver = bus.subscribe(KeyManagementEvent::KeyGenerated);

        // Publish a different event
        bus.publish(KeyManagementEvent::KeyEncrypted);

        // Ensure no event is received
        assert!(receiver.recv_timeout(Duration::from_millis(100)).is_err());
    }
} 