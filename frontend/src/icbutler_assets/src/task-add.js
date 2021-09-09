import {htmlHelper} from './base.js'


export class addTaskPage extends htmlHelper {
    constructor($parent) {
        super(null);

        this.HTML_ASK = `
        <div class="col-md-2">
        </div>
        <div class="col-md-8">
            <form> 
                <div class='form-group'> 
                    <label for='newTask'>Add task:</label> 
                    <textarea class='form-control' id='newTask' rows='7'></textarea> 
                </div>
                <button type='submit' class='btn btn-secondary button-submit'>Submit</button> 
                <button type='submit' class='btn btn-secondary button-cancel'>Cancel</button> 
            </form>
        </div>

        <div class="col-md-2">
        </div>
        `;

        this.$parent = $parent;
        this.createView();

        this.$newTask = this.$parent.find('#newTask');
        this.$submit = this.$parent.find('.button-submit');
        this.$cancel = this.$parent.find('.button-cancel');

        this.createHandlers();
    }

    destroy() {
        this.$parent.empty();
    }

    createHandlers() {
        this.$submit.click(async () => {
            let newTask = $.trim(this.$newTask.val());

            if (newTask.length > 0) {
                window.icButler.addTask(newTask);

                alert('New task is added!');
                this.$newTask.val('');
                window.icButler.loadTaskListPage();
            } else {
                alert('Please enter new task!')
            }
        });

        this.$cancel.click(async () => {
            window.icButler.loadTaskListPage();
        });
    }

    createView() {
        this.insertChildTop(this.$parent, this.HTML_ASK);
    }
}
