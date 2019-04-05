resource "aws_iam_role" "dino_park_fence_role" {
  name = "dino-park-fence-role-${var.environment}-${var.region}"

  assume_role_policy = <<EOF
{
   "Version": "2012-10-17",
   "Statement": [
     {
      "Effect": "Allow",
      "Principal": {
       "Service": "ec2.amazonaws.com"
      },
      "Action": "sts:AssumeRole"
     },
     {
      "Effect": "Allow",
      "Principal": {
        "AWS": "arn:aws:iam::${data.aws_caller_identity.current.account_id}:role/kubernetes-stage-us-west-220190207165215030100000005"
       },
       "Action": "sts:AssumeRole"
      }
   ]
}
EOF
}

resource "aws_iam_role_policy" "dino_park_fence_ssm_access" {
  name        = "dino-park-fence-ssm-access-${var.environment}-${var.region}"
  role        = "${aws_iam_role.dino_park_fence_role.id}"

  policy      = <<EOF
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Action": [
                "ssm:GetParameterHistory",
                "ssm:GetParametersByPath",
                "ssm:GetParameters",
                "ssm:GetParameter"
            ],
            "Resource": [
                "arn:aws:ssm:us-west-2:${data.aws_caller_identity.current.account_id}:parameter/iam/cis/development/*"
            ],
            "Effect": "Allow"
        },
        {
            "Action": [
                "kms:Decrypt"
            ],
            "Resource": [
                "arn:aws:kms:us-west-2:320464205386:key/ef00015d-739b-456d-a92f-482712af4f32"
            ],
            "Effect": "Allow"
        }
    ]
}
EOF
}
