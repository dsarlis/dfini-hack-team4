import type { Principal } from '@dfinity/principal';
export interface DfTask { 'id' : bigint, 'description' : string }
export interface _SERVICE {
  'addDfTask' : (arg_0: string) => Promise<string>,
  'addTask' : (arg_0: string) => Promise<string>,
  'getDfTask' : (arg_0: bigint) => Promise<DfTask>,
  'getDfTaskList' : () => Promise<Array<DfTask>>,
  'getTaskList' : () => Promise<Array<string>>,
  'greet' : (arg_0: string) => Promise<string>,
}
