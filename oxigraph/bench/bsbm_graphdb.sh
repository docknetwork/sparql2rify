#!/usr/bin/env bash

DATASET_SIZE=10000
cd bsbm-tools
./generate -fc -pc ${DATASET_SIZE} -s nt -fn "explore-${DATASET_SIZE}"
export JAVA_HOME=/usr/lib/jvm/java-8-openjdk-amd64
../graphdb-free-9.3.3/bin/graphdb -s -Dgraphdb.logger.root.level=WARN &
sleep 10
curl -f -X POST http://localhost:7200/rest/repositories -H 'Content-Type:application/json' -d '
{"id":"test","params":{"ruleset":{"label":"Ruleset","name":"ruleset","value":"empty"},"title":{"label":"Repository title","name":"title","value":"GraphDB Free repository"},"checkForInconsistencies":{"label":"Check for inconsistencies","name":"checkForInconsistencies","value":"false"},"disableSameAs":{"label":"Disable owl:sameAs","name":"disableSameAs","value":"true"},"baseURL":{"label":"Base URL","name":"baseURL","value":"http://example.org/owlim#"},"repositoryType":{"label":"Repository type","name":"repositoryType","value":"file-repository"},"id":{"label":"Repository ID","name":"id","value":"repo-test"},"storageFolder":{"label":"Storage folder","name":"storageFolder","value":"storage"}},"title":"Test","type":"free"}
'
curl -f -X PUT -H 'Content-Type:application/n-triples' --data-binary "@explore-${DATASET_SIZE}.nt" http://localhost:7200/repositories/test/statements
./testdriver -ucf usecases/explore/sparql.txt -o "../bsbm.explore.graphdb.${DATASET_SIZE}.9.3.3.xml" http://localhost:7200/repositories/test
./testdriver -ucf usecases/businessIntelligence/sparql.txt -o "../bsbm.businessIntelligence.graphdb.${DATASET_SIZE}.9.3.3.xml" http://localhost:7200/repositories/test
kill $!
sleep 5
rm -r ../graphdb-free-9.3.3/data
rm "explore-${DATASET_SIZE}.nt"
rm -r td_data
