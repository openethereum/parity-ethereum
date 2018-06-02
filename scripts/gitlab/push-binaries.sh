echo "__________Push binaries to AWS S3__________"
aws configure set aws_access_key_id $s3_key
aws configure set aws_secret_access_key $s3_secret
if [[ "$CI_BUILD_REF_NAME" = "master" || "$CI_BUILD_REF_NAME" = "beta" || "$CI_BUILD_REF_NAME" = "stable" || "$CI_BUILD_REF_NAME" = "nightly" ]];
then
  export S3_BUCKET=builds-parity-published;
else
  export S3_BUCKET=builds-parity;
fi
aws s3 rm --recursive s3://$S3_BUCKET/$CI_BUILD_REF_NAME/$BUILD_PLATFORM
aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$BUILD_PLATFORM/parity$S3WIN --body target/$PLATFORM/release/parity$S3WIN
aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$BUILD_PLATFORM/parity$S3WIN.md5 --body parity$S3WIN.md5
aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$BUILD_PLATFORM/parity$S3WIN.sha256 --body parity$S3WIN.sha256
aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$BUILD_PLATFORM/parity-evm$S3WIN --body target/$PLATFORM/release/parity-evm$S3WIN
aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$BUILD_PLATFORM/parity-evm$S3WIN.md5 --body parity-evm$S3WIN.md5
aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$BUILD_PLATFORM/parity-evm$S3WIN.sha256 --body parity-evm$S3WIN.sha256
aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$BUILD_PLATFORM/ethstore$S3WIN --body target/$PLATFORM/release/ethstore$S3WIN
aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$BUILD_PLATFORM/ethstore$S3WIN.md5 --body ethstore$S3WIN.md5
aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$BUILD_PLATFORM/ethstore$S3WIN.sha256 --body ethstore$S3WIN.sha256
aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$BUILD_PLATFORM/ethkey$S3WIN --body target/$PLATFORM/release/ethkey$S3WIN
aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$BUILD_PLATFORM/ethkey$S3WIN.md5 --body ethkey$S3WIN.md5
aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$BUILD_PLATFORM/ethkey$S3WIN.sha256 --body ethkey$S3WIN.sha256
aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$BUILD_PLATFORM/"parity_"$VER"_"$IDENT"_"$ARC"."$EXT --body "parity_"$VER"_"$IDENT"_"$ARC"."$EXT
aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$BUILD_PLATFORM/"parity_"$VER"_"$IDENT"_"$ARC"."$EXT".md5" --body "parity_"$VER"_"$IDENT"_"$ARC"."$EXT".md5"
aws s3api put-object --bucket $S3_BUCKET --key $CI_BUILD_REF_NAME/$BUILD_PLATFORM/"parity_"$VER"_"$IDENT"_"$ARC"."$EXT".sha256" --body "parity_"$VER"_"$IDENT"_"$ARC"."$EXT".sha256"
