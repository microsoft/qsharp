branch=main
for commit in $(git rev-list -n10 $branch)
do
  echo "benching commit" . $commit
  cargo criterion --history-id $commit
done
