use ic_cdk::api::{caller, time};
use ic_cdk::export::candid::{CandidType, Decode, Deserialize, Principal};
use ic_cdk_macros::{query, update};
use serde_bytes::ByteBuf;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    convert::TryFrom,
};

// The initial amount of tokens given to a newly registered principal.
const INITIAL_TOKENS: Amount = 1000;

// The cost of submitting a task. The main purpose is to prevent DoS attacks.
const SUBMISSION_COST: Amount = 1;

// The minimum duration that a task can stay open.
const MIN_DURATION: std::time::Duration = std::time::Duration::from_secs(60); // 1 minute

// The maximum duration that a task can stay open.
const MAX_DURATION: std::time::Duration = std::time::Duration::from_secs(60 * 60 * 24); // 1 day

// The maximum size of the payload of a task.
const MAX_TASK_PAYLOAD: usize = 10 * 1024; // 100 KiB

// The maximum number of answers that can be provided per task.
const MAX_NUMBER_ANSWERS: usize = 10;

// The maximum size of an answer's content.
const MAX_CONTENT_SIZE: usize = 10 * 1024; // 100 KiB

type AnswerId = u64;
type Content = ByteBuf;
type Duration = u64;
type TaskId = u64;
type TaskPayload = ByteBuf;
type Timestamp = u64;
type Amount = u64;

struct State {
    next_task_id: RefCell<TaskId>,
    tasks: RefCell<HashMap<TaskId, TaskInternal>>,
    answers: RefCell<HashMap<AnswerId, Answer>>,
    next_answer_id: RefCell<AnswerId>,
    ledger: RefCell<HashMap<Principal, Amount>>,
}

impl Default for State {
    fn default() -> Self {
        State {
            next_task_id: RefCell::new(0),
            tasks: RefCell::new(HashMap::default()),
            answers: RefCell::new(HashMap::default()),
            next_answer_id: RefCell::new(0),
            ledger: RefCell::new(HashMap::default()),
        }
    }
}

thread_local! {
    static STATE: State = State::default();
}

#[derive(Clone, Debug, CandidType, Deserialize)]
enum Choice {
    #[serde(rename = "yes")]
    Yes,
    #[serde(rename = "no")]
    No,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
enum Language {
    #[serde(rename = "chinese")]
    Chinese,
    #[serde(rename = "french")]
    French,
    #[serde(rename = "german")]
    German,
    #[serde(rename = "greek")]
    Greek,
    #[serde(rename = "hindi")]
    Hindi,
    #[serde(rename = "italian")]
    Italian,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
enum TaskType {
    #[serde(rename = "translate_text")]
    TranslateText,
    #[serde(rename = "edit_image")]
    EditImage,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
enum TaskStatus {
    #[serde(rename = "open")]
    Open,
    #[serde(rename = "closed")]
    Closed,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct TranslateTextInput {
    input: String,
    language: Language,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct Answer {
    submitter: Principal,
    submission_time: Timestamp,
    votes: Vec<Vote>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct Task {
    submitter: Principal,
    task_type: TaskType,
    payload: TaskPayload,
    deadline: Timestamp,
    reward: Amount,
    answers: Vec<Answer>,
    status: TaskStatus,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct ShortTask {
    id: TaskId,
    submitter: Principal,
    task_type: TaskType,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct Vote {
    voter: Principal,
    choice: Choice,
}

#[allow(dead_code)]
struct TaskInternal {
    submitter: Principal,
    task_type: TaskType,
    payload: TaskPayload,
    deadline: Timestamp,
    reward: Amount,
    answers: HashSet<AnswerId>,
    status: TaskStatus,
}

#[update]
fn register() {
    let caller = caller();

    STATE.with(|s| {
        let mut ledger = s.ledger.borrow_mut();
        if ledger.contains_key(&caller) {
            ic_cdk::trap(&format!("{} has already registered.", caller));
        }
        ledger.insert(caller, INITIAL_TOKENS);
    });
}

#[update]
fn submit_task(
    task_type: TaskType,
    payload: TaskPayload,
    duration: Duration,
    reward: Amount,
) -> TaskId {
    let caller = caller();

    STATE.with(|s| {
        let ledger = s.ledger.borrow();
        match ledger.get(&caller) {
            Some(balance) => {
                if *balance < SUBMISSION_COST {
                    ic_cdk::trap(&format!(
                        "{} has only {} tokens but {} are needed to submit a task.",
                        caller, balance, SUBMISSION_COST
                    ));
                }
            }
            None => {
                ic_cdk::trap(&format!("{} has not been registered yet.", caller));
            }
        }
    });

    match task_type {
        TaskType::TranslateText => {
            if payload.len() > MAX_TASK_PAYLOAD {
                ic_cdk::trap(&format!(
                    "Maximum size of payload is {} but {} was given.",
                    MAX_TASK_PAYLOAD,
                    payload.len()
                ));
            }

            if let Err(err) = Decode!(&payload, TranslateTextInput) {
                ic_cdk::trap(&format!("Invalid input for tranlating text: {}", err));
            };

            if duration < u64::try_from(MIN_DURATION.as_nanos()).unwrap() {
                ic_cdk::trap(&format!(
                    "Mininum duration for task is {:?}, but {:?} was given",
                    MIN_DURATION,
                    std::time::Duration::from_nanos(duration)
                ));
            }

            if duration > u64::try_from(MAX_DURATION.as_nanos()).unwrap() {
                ic_cdk::trap(&format!(
                    "Maximum duration for task is {:?}, but {:?} was given",
                    MAX_DURATION,
                    std::time::Duration::from_nanos(duration)
                ));
            }

            STATE.with(|s| {
                let mut ledger = s.ledger.borrow_mut();
                // Safe because we have checked that the caller is registered above.
                let balance = *ledger.get(&caller).unwrap();
                if balance < SUBMISSION_COST + reward {
                    ic_cdk::trap(&format!(
                        "{} has only {} tokens but {} were requested as a reward.",
                        caller,
                        balance - SUBMISSION_COST,
                        reward
                    ));
                }
                ledger.insert(caller, balance - SUBMISSION_COST - reward);
            });

            let task_id = STATE.with(|s| s.next_task_id.replace_with(|&mut old| old + 1));

            STATE.with(|s| {
                let mut tasks = s.tasks.borrow_mut();
                tasks.insert(
                    task_id,
                    TaskInternal {
                        submitter: caller,
                        task_type,
                        payload,
                        deadline: time() + duration,
                        reward,
                        answers: HashSet::new(),
                        status: TaskStatus::Open,
                    },
                );
            });

            task_id
        }
        TaskType::EditImage => ic_cdk::trap("Edit image use case is unimplemented!"),
    }
}

#[query]
fn get_task(_id: TaskId) -> Task {
    Task {
        submitter: Principal::anonymous(),
        task_type: TaskType::TranslateText,
        payload: ByteBuf::from(vec![]),
        deadline: 0,
        reward: 0,
        answers: vec![],
        status: TaskStatus::Open,
    }
}

#[query]
fn get_all_tasks() -> Vec<ShortTask> {
    vec![]
}

#[query]
fn get_balance() -> Amount {
    0
}

#[update]
fn answer_task(task_id: TaskId, content: Content) -> AnswerId {
    let caller = caller();
    STATE.with(|s| {
        let ledger = s.ledger.borrow();

        // Precondition: caller is a principal on the ledger
        if !ledger.contains_key(&caller) {
            ic_cdk::trap(&format!(
                "Principal {} cannot provide an answer as this is not a registered\
             user on the ledger.",
                caller
            ));
        }
    });

    STATE.with(|s| {
        let mut tasks = s.tasks.borrow_mut();
        let mut answers = s.answers.borrow_mut();

        match tasks.get_mut(&task_id) {
            // the task ID does not exist
            None => {
                ic_cdk::trap(&format!(
                    "Cannot provide an answer to task with ID {} as this task does not exist.",
                    task_id));
            }
            Some(task) => {
                // Precondition: there are less than max_answers for taskID
                if task.answers.len() >= MAX_NUMBER_ANSWERS {
                    ic_cdk::trap(&format!(
                        "Cannot provide an answer to task {} as the maximum number of {} answers is already \
                    reached.",
                        task_id, MAX_NUMBER_ANSWERS));
                }

                // Precondition: the deadline for the task has not expired
                if task.deadline < time() {
                    ic_cdk::trap(&format!(
                        "No new solution can be provided as the deadline for the task {} has already expired.",
                        task_id));
                }

                // Precondition: the solution's size is less than MAX_CONTENT_SIZE
                if content.len() > MAX_CONTENT_SIZE {
                    ic_cdk::trap(&format!(
                        "Maximum size of solution is {} but {} was given.",
                        MAX_CONTENT_SIZE, content.len()
                    ));
                }

                // Precondition: the caller hasn’t submitted an answer for this task
                for answer_id in task.answers.iter() {
                    match answers.get(answer_id){
                        Some(answer) => {
                            if caller == answer.submitter {
                                ic_cdk::trap(&format!(
                                    "The principal {} already submitted an answer for the task with ID {}.",
                                    caller, task_id));
                            }
                        },
                        // this is a case which should not occur, but let's catch it just to be sure
                        None => {
                            ic_cdk::trap(&format!(
                                "The answer with ID {} was listed in task with ID {} even though there is no\
                        such answer recorded.",
                                answer_id, task_id));
                        }
                    }
                }

                // Now that all the preconditions are met, make the new answer and submit it
                let answer_id = s.next_answer_id.replace_with(|&mut old| old + 1);
                task.answers.insert(answer_id);
                answers.insert(
                    answer_id,
                    Answer {
                        submitter: caller,
                        submission_time: time(),
                        votes: vec![],
                    },
                );
                answer_id
            }
        }
    })
}

#[update]
fn vote(_id: AnswerId, _choice: Choice) {}

#[export_name = "canister_heartbeat"]
fn hearbeat() {}

fn main() {}
