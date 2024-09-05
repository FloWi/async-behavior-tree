use anyhow::anyhow;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// inspired by @chamlis design from spacetraders discord

#[derive(Debug, Clone, Serialize)]
pub enum Behavior<A> {
    Action(A),
    Invert(Box<Behavior<A>>),
    Select(Vec<Behavior<A>>),
    Sequence(Vec<Behavior<A>>),
    // Success,
    // Run the action while the condition is successful or until the action returns a failure.
    // While {
    //     condition: Box<Behavior<A>>,
    //     action: Box<Behavior<A>>,
    // },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Response {
    Success,
    Running,
}

#[async_trait]
pub trait Actionable: Serialize + Clone + Send + Sync {
    type ActionError: From<anyhow::Error> + Send + Sync;
    type ActionArgs: Clone + Send + Sync;
    type ActionState: Send + Sync;

    async fn run(
        &self,
        args: &Self::ActionArgs,
        state: &mut Self::ActionState,
    ) -> Result<Response, Self::ActionError>;
}

#[async_trait]
impl<A> Actionable for Behavior<A>
where
    A: Actionable + Serialize,
{
    type ActionError = <A as Actionable>::ActionError;
    type ActionArgs = <A as Actionable>::ActionArgs;
    type ActionState = <A as Actionable>::ActionState;

    async fn run(
        &self,
        args: &Self::ActionArgs,
        state: &mut Self::ActionState,
    ) -> Result<Response, Self::ActionError> {
        match self {
            Behavior::Action(a) => {
                let result = a.run(args, state).await;
                result
            }
            Behavior::Invert(b) => {
                let result = b.run(args, state).await;
                match result {
                    Ok(r) => {
                        let inverted = match r {
                            Response::Success => {
                                Err(Self::ActionError::from(anyhow!("Inverted Ok")))
                            }
                            Response::Running => Ok(Response::Running),
                        };
                        inverted
                    }
                    Err(_) => Ok(Response::Success),
                }
            }
            Behavior::Select(behaviors) => {
                for b in behaviors {
                    let result = b.run(args, state).await;
                    match result {
                        Ok(r) => return Ok(r),
                        Err(_) => continue,
                    }
                }
                Err(Self::ActionError::from(anyhow!("No behavior successful")))
            } // Behavior::Sequence(_) => {}
            // Behavior::Success => {}
            // Behavior::While { .. } => {}
            Behavior::Sequence(behaviors) => {
                for b in behaviors {
                    let result = b.run(args, state).await;
                    match result {
                        Ok(r) => continue,
                        Err(_) => {
                            return Err(Self::ActionError::from(anyhow!("one behavior failed")))
                        }
                    }
                }
                Ok(Response::Success)
            }
        }
    }
}
