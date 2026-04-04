#!/bin/bash
# Makefile to Taskfile Migration Template

echo "Migrating Makefile to Taskfile..."

if [ ! -f Makefile ] && [ ! -f makefile ]; then
    echo "No Makefile found"
    exit 1
fi

cat > Taskfile.yml << 'TASKFILE'
version: '3'

tasks:
  default:
    desc: List all tasks
    cmds:
      - task --list
TASKFILE

echo "" >> Taskfile.yml

# Extract targets from Makefile
grep -E '^[a-zA-Z_-]+:.*$' Makefile 2>/dev/null | grep -v '.PHONY' | while read line; do
    target=$(echo "$line" | cut -d: -f1)
    echo "  $target:" >> Taskfile.yml
    echo "    desc: $target task" >> Taskfile.yml
    echo "    cmds:" >> Taskfile.yml
    echo "      - echo 'Add commands here'" >> Taskfile.yml
    echo "" >> Taskfile.yml
done

mv Makefile Makefile.backup.$(date +%Y%m%d)
echo "Makefile to Taskfile migration complete"
