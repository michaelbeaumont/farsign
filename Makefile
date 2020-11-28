SHELL := /bin/bash
markary := ../markary

pages:
	mkdir -p docs; \
	tmpdef=$$(mktemp); \
	MARKARY=${markary} envsubst < ${markary}/markary.yaml > $${tmpdef}; \
	for post in posts/*.md; do \
	  pagename="$$(basename $${post%.md})"; \
	  pandoc $${post} \
	    --include-before-body=header.html \
	    --css=header.css \
	    --defaults=$${tmpdef} \
	    -o "docs/$${pagename}.html"; \
	done

watch:
	while inotifywait -r -e move_self posts/*.md; do make pages; done

