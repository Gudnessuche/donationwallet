// AUTO GENERATED FILE, DO NOT EDIT.
// Generated by `flutter_rust_bridge`@ 1.82.4.
// ignore_for_file: non_constant_identifier_names, unused_element, duplicate_ignore, directives_ordering, curly_braces_in_flow_control_structures, unnecessary_lambdas, slash_for_doc_comments, prefer_const_literals_to_create_immutables, implicit_dynamic_list_literal, duplicate_import, unused_import, unnecessary_import, prefer_single_quotes, prefer_const_constructors, use_super_parameters, always_use_package_imports, annotate_overrides, invalid_use_of_protected_member, constant_identifier_names, invalid_use_of_internal_member, prefer_is_empty, unnecessary_const

import 'dart:convert';
import 'dart:async';
import 'package:meta/meta.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:uuid/uuid.dart';

abstract class Rust {
  Stream<LogEntry> createLogStream({dynamic hint});

  FlutterRustBridgeTaskConstMeta get kCreateLogStreamConstMeta;

  Stream<int> createAmountStream({dynamic hint});

  FlutterRustBridgeTaskConstMeta get kCreateAmountStreamConstMeta;

  Stream<ScanProgress> createScanProgressStream({dynamic hint});

  FlutterRustBridgeTaskConstMeta get kCreateScanProgressStreamConstMeta;

  Future<String> setup(
      {required String label,
      required String network,
      String? seedWords,
      dynamic hint});

  FlutterRustBridgeTaskConstMeta get kSetupConstMeta;

  Future<void> startNakamoto({dynamic hint});

  FlutterRustBridgeTaskConstMeta get kStartNakamotoConstMeta;

  Future<void> restartNakamoto({dynamic hint});

  FlutterRustBridgeTaskConstMeta get kRestartNakamotoConstMeta;

  Future<int> getPeerCount({dynamic hint});

  FlutterRustBridgeTaskConstMeta get kGetPeerCountConstMeta;

  Future<void> scanNextNBlocks(
      {required String blob, required int n, dynamic hint});

  FlutterRustBridgeTaskConstMeta get kScanNextNBlocksConstMeta;

  Future<void> scanToTip({required String blob, dynamic hint});

  FlutterRustBridgeTaskConstMeta get kScanToTipConstMeta;

  Future<ScanStatus> getWalletInfo({required String blob, dynamic hint});

  FlutterRustBridgeTaskConstMeta get kGetWalletInfoConstMeta;

  Future<int> getWalletBalance({required String blob, dynamic hint});

  FlutterRustBridgeTaskConstMeta get kGetWalletBalanceConstMeta;

  Future<String> getReceivingAddress({required String blob, dynamic hint});

  FlutterRustBridgeTaskConstMeta get kGetReceivingAddressConstMeta;
}

class LogEntry {
  final String msg;

  const LogEntry({
    required this.msg,
  });
}

class ScanProgress {
  final int start;
  final int current;
  final int end;

  const ScanProgress({
    required this.start,
    required this.current,
    required this.end,
  });
}

class ScanStatus {
  final int scanHeight;
  final int blockTip;

  const ScanStatus({
    required this.scanHeight,
    required this.blockTip,
  });
}
