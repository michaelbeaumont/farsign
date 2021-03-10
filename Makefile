SHELL := /bin/bash
markary := ../markary
markary.yaml := $(markary)/markary.yaml
tmpconfig := tmpconfig.yaml

all: $(patsubst posts/%.md,docs/%.html,$(wildcard posts/*.md))

docs:
	mkdir docs

$(tmpconfig): $(markary.yaml)
	MARKARY=${markary} envsubst < "$<" > $(tmpconfig)

docs/%.html: posts/%.md $(markary.yaml) header.html header.css docs/media | docs $(tmpconfig)
	pandoc "$<" \
	  --include-before-body=header.html \
	  --resource-path=.:docs \
	  --css=header.css \
	  --defaults=$(tmpconfig) \
	  -o "$@"

watch:
	while inotifywait -r -e move_self posts/*.md; do make; done
