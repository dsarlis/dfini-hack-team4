import {htmlHelper} from './base.js'


export class listTasksPage extends htmlHelper {
    constructor(taskList, $parent) {
        super(null);

        this.HTML_TASK_LIST = `
        <div class="col-md-2">
            All Tasks
        </div>
        <div class="col-md-8">
            <div class="list-wrapper" id="list-wrapper">
                <ul>
                </ul>
            </div>
        </div>

        <div class="col-md-2">
            <button id="add-task">Add Task</button>
        </div>
        `;

        this.HTML_LIST_ITEM = ({url, title, author_list, publish_time}) => `
        <li data-value="${url}">
          <h3>${title}</h3>
          
          <div class="article-info">
            <div class="publish-time">${publish_time}</div>
            <div class="author-list">${author_list}</div>
          </div>
        </li>
        `;


        this.$parent = $parent;
        this.taskList = taskList;

        this.$listWrapper = null;
        this.$ul = null;
        this.$addTask = null;
        this.createView();
    }

    destroy() {
        this.$parent.empty();
    }

    createHandlers() {
        this.$addTask.off('click');
        this.$addTask.click(async () => {
          // Redirect to add task page.
          window.icButler.addTaskPage();
        });

        this.$ul.find('ul > li').click($.proxy(this.event_itemClick, this));
    }
    
    event_itemClick(evt) {
        let taskId = $(evt.target).parent().attr('data-value');

        // Redirect to task detail page and and call canister to get task details.
        window.icButler.taskDetailPage(taskId);
    }

    createView() {
        this.insertChildTop(this.$parent, this.HTML_TASK_LIST);
        this.$listWrapper = this.$parent.find('.list-wrapper');
        this.$ul = this.$listWrapper.find('ul');

        for(let i = 0; i < this.taskList.length; i++) {
            let item = [];
            item.push({url: this.taskList[i].id,
                       publish_time: '09-05-2021',
                       author_list: 'Author',
                       title: this.taskList[i].description});
            let addItem = item.map(this.HTML_LIST_ITEM).join('');
            this.$ul.prepend(addItem);

            let $addItem = this.$ul.last();
            let $title = $addItem.find('h3');
            $title.click($.proxy(this.event_itemClick, this));
        }

        this.$addTask = this.$parent.find('#add-task');
        this.createHandlers();
    }
}
