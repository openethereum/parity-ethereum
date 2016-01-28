#!/bin/sh
echo "#!/bin/sh\ncargo test" >> ./.git/hooks/pre-push
chmod +x ./.git/hooks/pre-push
