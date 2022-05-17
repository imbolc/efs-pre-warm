EFS pre-warm
============

EFS with activated "infrequent access policy" archives old data.
To warm them back this program recursively reads all the files created more than `ia_days` days ago
starting from the current folder.
