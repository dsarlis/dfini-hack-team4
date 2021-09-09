import Array "mo:base/Array";
import Nat "mo:base/Nat";
import Debug "mo:base/Debug";


actor {
    type DfTask = {
        id: Nat;
        description: Text;
    };

    var dfTaskList : [DfTask] = [];
    var nextId : Nat = 1;

    public func addDfTask(task : Text) : async Text {
        let addTask : DfTask = {
            id = nextId;
            description = task;
        };

        dfTaskList := Array.append(dfTaskList, [addTask]);
        nextId += 1;

        return Nat.toText(nextId);
    };

    public func getDfTaskList() : async [DfTask] {
        return dfTaskList;
    };

    public func getDfTaskAnswers(taskId: Nat) : async [Text] {
        // TODO: It should return list of answers associated with the task.
        let answerList: [Text] = [];

        return answerList;
    };

    public func getDfTask(taskId: Nat) : async DfTask {
        // TODO: search task by id.
        let findTask : DfTask = {
            id = 1;
            description = "dummy task";
        };

        return findTask;
    };
};
