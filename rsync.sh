#!/bin/sh
# must be installed in both sides!!!
#
#/usr/bin/rsync -rv --dry-run workspace mvcorrea@10.10.10.217:~/podman/TI0162-Internet-das-Coisas-PRJ
DIR=~/Private/20250626_rust_projects/TI0162-Internet-das-Coisas-PRJ
echo "from: "$DIR
#/usr/bin/rsync -rv --exclude 'target/' $PATH/workspace mvcorrea@10.10.10.217:~/podman/TI0162-Internet-das-Coisas-PRJ
/usr/bin/rsync -rv --delete --exclude 'target/'  ${DIR}/workspace mvcorrea@10.10.10.217:~/podman/TI0162-Internet-das-Coisas-PRJ
