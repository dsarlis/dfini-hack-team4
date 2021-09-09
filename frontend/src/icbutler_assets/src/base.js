export class htmlHelper {

    constructor(options) {
        let defaultOptions = {
            base_class: null,
            notify_destroy: null,
            notify_create: null,
        }

        // Set options
        this.options = $.extend(defaultOptions, options);
    }

    insertChildTop($anchorItem, newItem) {
        let $newItem = $(newItem);
        $anchorItem.prepend(newItem);
        return $newItem;
    }

    insertChildBottom($anchorItem, newItem) {
       return $(newItem).appendTo($anchorItem);
    }
}
