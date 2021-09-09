import {htmlHelper} from './base.js'


export class taskDetailPage extends htmlHelper {
    constructor(taskDetail, $parent) {
        super(null);

        this.HTML_TASK_TITLE = ({title}) =>  `
        <div class="col-md-2">
        </div>
        <div class="col-md-8">
            <article>
                <section class="title">
                <header>${title}</header>
                    <div class="task-info">
                    </div>
                </div>
                </section>
            </article>
        </div>
        <div class="col-md-2">
        </div>
        `;

        this.HTML_TASK_INFO = ({authors, publish_time}) => `
        <div class="article-info">
          <div class="publish-time">${publish_time}</div>
          <div class="authors">${authors}</div>
        </div>
        `;

        this.HTML_PARAGRAPH = ({content}) => `
            <p>${content}</p>
        `;

        this.taskDetail = taskDetail;
        this.$parent = $parent;
        this.createView();
    }

    destroy() {
        this.$parent.empty();
    }

    createView() {
        // Task title
        let item = [];
        item.push({title: this.taskDetail.description});
        let addItem = item.map(this.HTML_TASK_TITLE).join('');
        this.$parent.append(addItem);

        let $article = this.$parent.find('article');
        let $taskInfo = $article.find('.task-info');

        // Task info
        item.push({authors: 'Author', 
                   publish_time: '2021-03-6'});
        addItem = item.map(this.HTML_TASK_INFO).join('');
        $(addItem).appendTo($taskInfo);

        // Task details
        item = [];
        item.push({section_name: 'Abstract', 
                   id: 'abstract'});
        addItem = item.map(this.HTML_SECTION_HEAD).join('');
        $(addItem).appendTo($article);

        this.createHandlers();
    }
}
