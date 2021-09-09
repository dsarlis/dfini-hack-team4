import {icbutler} from "../../declarations/icbutler";
import {htmlHelper} from './base.js'
import {listTasksPage} from './task-list.js'
import {addTaskPage} from './task-add.js'
import {taskDetailPage} from './task-detail.js'


export class viewControl extends htmlHelper {
  constructor() {
      super();

      this.$icButler = $('.div-icbutler');
      this.$content = this.$icButler.find('.div-content-section');

      this.pageTaskList = null;
      this.pageAddTask = null;
      this.pageTaskDetail = null;

      this.loadTaskListPage();
  }

  async addTask(task) {
    return await icbutler.addDfTask(task);
  } 
  
  async taskDetailPage(taskId) {
    this.clearCurrentPage();

    let taskDetail = await icbutler.getDfTask(parseInt(taskId));
    this.pageTaskDetail = new taskDetailPage(taskDetail, this.$content);
  }

  async addTaskPage() {
    this.clearCurrentPage();

    let taskArray = await icbutler.getDfTaskList();
    this.pageAddTask = new addTaskPage(this.$content);
  }

  clearCurrentPage() {
    if (this.pageTaskList) {
      this.pageTaskList.destroy();
      this.pageTaskList = null;
    }

    if (this.pageAddTask) {
      this.pageAddTask.destroy();
      this.pageAddTask = null;
    }

    if (this.pageTaskDetail) {
      this.pageTaskDetail.destroy();
      this.pageTaskDetail = null;
    }
  }

  async loadTaskListPage() {
    this.clearCurrentPage();
    let taskArray = await icbutler.getDfTaskList();
    this.pageTaskList = new listTasksPage(taskArray, this.$content);
  }
}


$(document).ready(async () => {
  window.icButler = new viewControl();
});
