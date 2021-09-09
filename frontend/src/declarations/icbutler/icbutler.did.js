export const idlFactory = ({ IDL }) => {
  const DfTask = IDL.Record({ 'id' : IDL.Nat, 'description' : IDL.Text });
  return IDL.Service({
    'addDfTask' : IDL.Func([IDL.Text], [IDL.Text], []),
    'addTask' : IDL.Func([IDL.Text], [IDL.Text], []),
    'getDfTask' : IDL.Func([IDL.Nat], [DfTask], []),
    'getDfTaskList' : IDL.Func([], [IDL.Vec(DfTask)], []),
    'getTaskList' : IDL.Func([], [IDL.Vec(IDL.Text)], []),
    'greet' : IDL.Func([IDL.Text], [IDL.Text], []),
  });
};
export const init = ({ IDL }) => { return []; };
